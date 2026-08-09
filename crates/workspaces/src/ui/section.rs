//! The bar section for workspaces and the app menu: the frontmost-app chip
//! with its rail/items popover, followed by the workspace pills. Talks only
//! to the ports via the app layer; the shell composes concrete adapters in
//! `impls` and reacts to `SectionEvent` for window sizing.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use base_gpui::popover::{
    PopoverAlign, PopoverHandle, PopoverPopup, PopoverPortal, PopoverPositioner, PopoverRoot,
    PopoverSide, PopoverTrigger, create_popover_handle,
};
use base_gpui::toggle::Toggle;
use base_gpui::toggle_group::ToggleGroup;
use gpui::{
    AnyElement, Context, EventEmitter, FontWeight, Image, ImageFormat, Window, div, img,
    prelude::*, px,
};

use crate::app::bar::{self, BarModel};
use crate::core::menu::MenuEntry;
use crate::core::workspace::{Workspace, WorkspaceId};
use crate::ui::palette;

/// Vertical room, in pixels, the shell must clear below the bar row while the
/// app-menu popover is open.
pub const POPUP_EXTENT: f64 = 340.0;

pub enum SectionEvent {
    /// The section needs `extent` pixels of window below the bar row
    /// (0 = collapsed back to the bare bar).
    PopupExtentChanged { extent: f64 },
}

pub struct WorkspacesSection {
    service: Arc<bar::BarService>,
    model: BarModel,
    /// Decoded-image handles per icon key. Built once per key so gpui's image
    /// cache sees a stable image id instead of re-decoding every frame.
    icon_images: HashMap<String, Arc<Image>>,
    /// Index of the rail entry whose items fill the right pane.
    selected_menu: usize,
    /// Last extent sent to the shell, to dedupe events.
    popup_extent: f64,
    popover: PopoverHandle<()>,
}

impl EventEmitter<SectionEvent> for WorkspacesSection {}

impl WorkspacesSection {
    pub fn new(service: Arc<bar::BarService>, cx: &mut Context<Self>) -> Self {
        let section = Self {
            service: service.clone(),
            model: BarModel::default(),
            icon_images: HashMap::new(),
            selected_menu: 0,
            popup_extent: 0.0,
            popover: create_popover_handle(),
        };

        cx.spawn(async move |this, cx| {
            let mut initial = true;
            loop {
                let service = service.clone();
                let loaded = cx
                    .background_executor()
                    .spawn(async move {
                        if initial {
                            service.load()
                        } else {
                            service.poll()
                        }
                    })
                    .await;
                initial = false;
                let alive = this.update(cx, |section, cx| {
                    // A failed observation (e.g. aerospace not running) keeps
                    // the previous model; the bar degrades, never blocks.
                    if let Ok(model) = loaded {
                        section.apply(model, cx);
                    }
                });
                if alive.is_err() {
                    break;
                }
            }
        })
        .detach();

        section
    }

    fn apply(&mut self, model: BarModel, cx: &mut Context<Self>) {
        if self.model == model {
            return;
        }
        for (key, png) in &model.icons {
            self.icon_images
                .entry(key.clone())
                .or_insert_with(|| Arc::new(Image::from_bytes(ImageFormat::Png, png.to_vec())));
        }
        self.selected_menu = self.selected_menu.min(model.menus.len().saturating_sub(1));
        self.model = model;
        cx.notify();
    }

    /// Tell the shell how much window the section needs below the bar row.
    fn set_popup_extent(&mut self, extent: f64, cx: &mut Context<Self>) {
        if self.popup_extent == extent {
            return;
        }
        self.popup_extent = extent;
        cx.emit(SectionEvent::PopupExtentChanged { extent });
    }

    fn focus(&mut self, workspace: WorkspaceId, cx: &mut Context<Self>) {
        let service = self.service.clone();
        cx.spawn(async move |this, cx| {
            let loaded = cx
                .background_executor()
                .spawn(async move { service.focus(&workspace) })
                .await;
            if let Ok(model) = loaded {
                this.update(cx, |section, cx| section.apply(model, cx)).ok();
            }
        })
        .detach();
    }

    fn activate_item(
        &mut self,
        app_name: String,
        menu_title: String,
        item_index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let popover = self.popover.clone();
        window.defer(cx, move |window, cx| {
            popover.close(window, cx);
        });
        let service = self.service.clone();
        cx.background_executor()
            .spawn(async move {
                let _ = service.activate(&app_name, &menu_title, item_index);
            })
            .detach();
    }

    fn render_chip(&self, app_name: String, cx: &mut Context<Self>) -> AnyElement {
        let on_open_change = {
            let entity = cx.entity().downgrade();
            move |open: bool, _details: &mut _, _window: &mut Window, cx: &mut gpui::App| {
                entity
                    .update(cx, |section: &mut Self, cx| {
                        section.selected_menu = 0;
                        section.set_popup_extent(if open { POPUP_EXTENT } else { 0.0 }, cx);
                        cx.notify();
                    })
                    .ok();
            }
        };

        let popover = PopoverRoot::<()>::new()
            .id("app-menu")
            .handle(self.popover.clone())
            .on_open_change(on_open_change)
            .child(
                PopoverTrigger::new()
                    .id("app-menu-trigger")
                    .open_on_hover(true)
                    .close_delay(Duration::from_millis(120))
                    .aria_label(app_name.clone())
                    .flex()
                    .items_center()
                    .h(px(22.))
                    .px(px(8.))
                    .rounded(px(6.))
                    .border_1()
                    .text_size(px(12.5))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(palette::text_bright())
                    .whitespace_nowrap()
                    .style_with_state(|state, trigger| {
                        if state.open {
                            trigger.bg(palette::raise()).border_color(palette::accent())
                        } else {
                            trigger
                                .bg(palette::raise_faint())
                                .border_color(palette::transparent())
                                .hover(|style| style.bg(palette::raise()))
                        }
                    })
                    .child(app_name.clone()),
            )
            .child(
                PopoverPortal::new().child(
                    PopoverPositioner::new()
                        .side(PopoverSide::Bottom)
                        .align(PopoverAlign::Start)
                        .side_offset(px(8.))
                        .align_offset(px(-12.))
                        .collision_padding(px(0.))
                        .child(
                            PopoverPopup::new()
                                .id("app-menu-popup")
                                .w(px(428.))
                                .h(px(320.))
                                .p(px(6.))
                                .rounded(px(12.))
                                .bg(palette::bar())
                                .border_1()
                                .border_color(palette::border())
                                .shadow_lg()
                                .flex()
                                .items_start()
                                .child_any(self.render_menu_rail(cx))
                                .child_any(self.render_menu_items(app_name, cx)),
                        ),
                ),
            );

        div()
            .id("app-menu-hover")
            .on_hover({
                let popover = self.popover.clone();
                cx.listener(move |_section, hovered: &bool, window, cx| {
                    if *hovered {
                        let popover = popover.clone();
                        window.defer(cx, move |window, cx| {
                            popover.open("app-menu-trigger", window, cx);
                        });
                    }
                })
            })
            .child(popover)
            .into_any_element()
    }

    fn render_menu_rail(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(128.))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .gap(px(1.))
            .pr(px(6.))
            .border_r_1()
            .border_color(palette::border())
            .children(self.model.menus.iter().enumerate().map(|(index, menu)| {
                let selected = index == self.selected_menu;
                div()
                    .id(("app-menu-rail", index))
                    .px(px(9.))
                    .py(px(6.))
                    .rounded(px(6.))
                    .text_size(px(12.5))
                    .cursor_pointer()
                    .when(selected, |rail_entry| rail_entry.bg(palette::raise()))
                    .text_color(if selected {
                        palette::text_bright()
                    } else {
                        palette::text()
                    })
                    .on_hover(cx.listener(move |section, hovered: &bool, _window, cx| {
                        if *hovered && section.selected_menu != index {
                            section.selected_menu = index;
                            cx.notify();
                        }
                    }))
                    .child(menu.title.clone())
            }))
    }

    fn render_menu_items(&self, app_name: String, cx: &mut Context<Self>) -> impl IntoElement {
        let entries = self
            .model
            .menus
            .get(self.selected_menu)
            .map(|menu| (menu.title.clone(), menu.entries.clone()))
            .into_iter()
            .flat_map(|(menu_title, entries)| {
                entries
                    .into_iter()
                    .enumerate()
                    .map(move |(row, entry)| (menu_title.clone(), row, entry))
                    .collect::<Vec<_>>()
            });

        div()
            .id("app-menu-items")
            .flex_1()
            .h_full()
            .min_w_0()
            .overflow_y_scroll()
            .pl(px(6.))
            .flex()
            .flex_col()
            .children(entries.map(|(menu_title, row, entry)| {
                match entry {
                    MenuEntry::Separator => div()
                        .h(px(9.))
                        .flex_none()
                        .flex()
                        .items_center()
                        .child(div().h(px(1.)).w_full().bg(palette::raise()))
                        .into_any_element(),
                    MenuEntry::Item(spec) => {
                        let app_name = app_name.clone();
                        div()
                            .id(format!("app-menu-item-{menu_title}-{row}"))
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .h(px(26.))
                            .flex_none()
                            .px(px(8.))
                            .rounded(px(6.))
                            .text_size(px(12.5))
                            .text_color(if spec.enabled {
                                palette::text()
                            } else {
                                palette::muted()
                            })
                            .child(
                                div()
                                    .w(px(14.))
                                    .flex_none()
                                    .text_center()
                                    .child(if spec.checked { "✓" } else { "" }),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .whitespace_nowrap()
                                    .overflow_hidden()
                                    .child(spec.label.clone()),
                            )
                            .children(spec.shortcut.clone().map(|shortcut| {
                                div()
                                    .ml_auto()
                                    .text_size(px(10.5))
                                    .text_color(palette::muted())
                                    .child(shortcut)
                            }))
                            .when(spec.enabled, |item| {
                                item.cursor_pointer()
                                    .hover(|style| {
                                        style.bg(palette::accent()).text_color(palette::ink())
                                    })
                                    .on_click(cx.listener(move |section, _event, window, cx| {
                                        section.activate_item(
                                            app_name.clone(),
                                            menu_title.clone(),
                                            row,
                                            window,
                                            cx,
                                        );
                                    }))
                            })
                            .into_any_element()
                    }
                }
            }))
    }

    fn render_pills(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let focused = self.model.snapshot.focused.clone();
        let on_value_change = {
            let entity = cx.entity().downgrade();
            move |values: &[String], _details: &mut _, _window: &mut Window, cx: &mut gpui::App| {
                let Some(selected) = values.first() else {
                    return;
                };
                let workspace = WorkspaceId::new(selected.clone());
                entity
                    .update(cx, |section: &mut Self, cx| section.focus(workspace, cx))
                    .ok();
            }
        };

        ToggleGroup::<String>::new()
            .id("workspace-pills")
            .aria_label("Workspaces")
            .multiple(false)
            .value(
                focused
                    .iter()
                    .map(|workspace| workspace.as_str().to_owned())
                    .collect(),
            )
            .on_value_change(on_value_change)
            .flex()
            .items_center()
            .gap(px(4.))
            .children(self.model.snapshot.workspaces.iter().enumerate().map(
                |(index, workspace)| {
                    render_pill(
                        index,
                        workspace,
                        focused.as_ref() == Some(&workspace.id),
                        &self.icon_images,
                    )
                },
            ))
    }
}

fn render_pill(
    index: usize,
    workspace: &Workspace,
    active: bool,
    icon_images: &HashMap<String, Arc<Image>>,
) -> Toggle<String> {
    let number_color = if active {
        palette::hue(index)
    } else {
        palette::hue_faint(index)
    };

    let number = div()
        .min_w(px(9.))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(10.5))
        .font_weight(FontWeight::BOLD)
        .text_color(number_color)
        .child(workspace.id.to_string());
    Toggle::<String>::new()
        .id(format!("workspace-pill-{}", workspace.id))
        .value(workspace.id.as_str().to_owned())
        .aria_label(format!("Workspace {}", workspace.id))
        .flex()
        .items_center()
        .gap(px(4.))
        .h(px(22.))
        .pl(px(4.))
        .pr(px(6.))
        .rounded(px(6.))
        .border_1()
        .style_with_state(|state, pill| {
            if state.pressed {
                pill.bg(palette::raise())
                    .border_color(palette::active_border())
            } else {
                pill.bg(palette::raise_faint())
                    .border_color(palette::transparent())
            }
        })
        .child(number)
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(2.))
                .children(workspace.tiles.iter().map(|tile| {
                    match icon_images.get(&tile.icon_key) {
                        Some(image) => img(image.clone())
                            .size(px(15.))
                            .rounded(px(4.))
                            .when(!active, |tile_element| tile_element.opacity(0.6))
                            .into_any_element(),
                        None => div()
                            .size(px(15.))
                            .rounded(px(4.))
                            .bg(palette::hue(tile.hue_slot))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(8.5))
                            .font_weight(FontWeight::BOLD)
                            .text_color(palette::ink())
                            .when(!active, |tile_element| tile_element.opacity(0.6))
                            .child(tile.initial.clone())
                            .into_any_element(),
                    }
                }))
                .when(workspace.tiles.is_empty(), |tiles| {
                    tiles.child(
                        div()
                            .size(px(15.))
                            .rounded(px(4.))
                            .border_1()
                            .border_dashed()
                            .border_color(palette::muted()),
                    )
                }),
        )
}

impl Render for WorkspacesSection {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chip = self
            .model
            .snapshot
            .frontmost_app
            .clone()
            .filter(|_| !self.model.menus.is_empty())
            .map(|app_name| self.render_chip(app_name, cx));

        div()
            .flex()
            .items_center()
            .gap(px(6.))
            .children(chip)
            .child(self.render_pills(cx))
    }
}
