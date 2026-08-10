//! The themer: a four-stripe swatch button opening the base16 theme picker
//! with appearance toggle, palette list, typeface sets, and accent slots.
//! Placeholder-only for now — selections are local UI state; the theme engine
//! (phase 2) will own applying them.

use base_gpui::popover::{
    PopoverAlign, PopoverHandle, PopoverPopup, PopoverPortal, PopoverPositioner, PopoverRoot,
    PopoverSide, PopoverTrigger, create_popover_handle,
};
use base_gpui::toggle::Toggle;
use base_gpui::toggle_group::ToggleGroup;
use gpui::{Context, EventEmitter, FontWeight, Rgba, Window, div, prelude::*, px, rgb};

use crate::core::catalog::{self, ACCENT_SLOTS};
use crate::core::{palette, typography};

/// Vertical room, in pixels, the shell must clear below the bar row while the
/// theme popover is open.
pub const POPUP_EXTENT: f64 = 540.0;

pub enum SectionEvent {
    /// The section needs `extent` pixels of window below the bar row
    /// (0 = collapsed back to the bare bar).
    PopupExtentChanged { extent: f64 },
}

pub struct ThemeSection {
    light: bool,
    selected_theme: usize,
    selected_font: usize,
    selected_accent: usize,
    open: bool,
    popup_extent: f64,
    popover: PopoverHandle<()>,
}

impl EventEmitter<SectionEvent> for ThemeSection {}

impl ThemeSection {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            light: false,
            selected_theme: 0,
            selected_font: 0,
            selected_accent: 4,
            open: false,
            popup_extent: 0.0,
            popover: create_popover_handle(),
        }
    }

    fn set_popup_extent(&mut self, extent: f64, cx: &mut Context<Self>) {
        if self.popup_extent == extent {
            return;
        }
        self.popup_extent = extent;
        cx.emit(SectionEvent::PopupExtentChanged { extent });
    }

    fn render_appearance_toggle(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let on_value_change = {
            let entity = cx.entity().downgrade();
            move |values: &[String], _details: &mut _, _window: &mut Window, cx: &mut gpui::App| {
                let Some(selected) = values.first() else {
                    return;
                };
                let light = selected == "light";
                entity
                    .update(cx, |section: &mut Self, cx| {
                        section.light = light;
                        cx.notify();
                    })
                    .ok();
            }
        };

        ToggleGroup::<String>::new()
            .id("theme-appearance")
            .aria_label("Appearance")
            .multiple(false)
            .value(vec![if self.light { "light" } else { "dark" }.to_owned()])
            .on_value_change(on_value_change)
            .flex()
            .p(px(2.))
            .rounded(px(6.))
            .bg(palette::raise())
            .gap(px(2.))
            .children(
                [("dark", "DARK"), ("light", "LIGHT")].map(|(value, label)| {
                    Toggle::<String>::new()
                        .id(format!("theme-appearance-{value}"))
                        .value(value.to_owned())
                        .aria_label(label)
                        .px(px(10.))
                        .py(px(4.))
                        .rounded(px(6.))
                        .font_family(typography::mono())
                        .text_size(px(9.5))
                        .style_with_state(|state, toggle| {
                            if state.pressed {
                                toggle.bg(palette::accent()).text_color(palette::ink())
                            } else {
                                toggle
                                    .bg(palette::transparent())
                                    .text_color(palette::subtle())
                            }
                        })
                        .child(label)
                }),
            )
    }

    fn render_theme_list(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("theme-list")
            .max_h(px(300.))
            .overflow_y_scroll()
            .mt(px(12.))
            .flex()
            .flex_col()
            .gap(px(3.))
            .children(catalog::catalog().iter().enumerate().map(|(index, entry)| {
                let selected = index == self.selected_theme;
                let dark_only = entry.light.is_none();
                let variant = if dark_only {
                    "dark only"
                } else if self.light {
                    "light"
                } else {
                    "dark"
                };
                div()
                    .id(("theme-card", index))
                    .flex_none()
                    .p(px(8.))
                    .rounded(px(8.))
                    .border_1()
                    .border_color(if selected {
                        palette::accent()
                    } else {
                        palette::border()
                    })
                    .bg(if selected {
                        palette::raise()
                    } else {
                        palette::transparent()
                    })
                    .when(self.light && dark_only, |card| card.opacity(0.55))
                    .cursor_pointer()
                    .hover(|style| style.border_color(palette::accent()))
                    .on_click(cx.listener(move |section, _event, _window, cx| {
                        section.selected_theme = index;
                        cx.notify();
                    }))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(
                                div()
                                    .text_size(px(12.5))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(if selected {
                                        palette::text_bright()
                                    } else {
                                        palette::text()
                                    })
                                    .child(entry.name),
                            )
                            .child(
                                div()
                                    .font_family(typography::mono())
                                    .text_size(px(9.5))
                                    .text_color(palette::muted())
                                    .child(variant),
                            )
                            .when(selected, |header| {
                                header.child(
                                    div()
                                        .ml_auto()
                                        .font_family(typography::mono())
                                        .text_size(px(10.))
                                        .text_color(palette::accent())
                                        .child("●"),
                                )
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .mt(px(8.))
                            .h(px(15.))
                            .rounded(px(4.))
                            .overflow_hidden()
                            .children(
                                entry
                                    .swatches(self.light)
                                    .iter()
                                    .map(|&color| div().flex_1().bg(rgb(color))),
                            ),
                    )
            }))
    }

    fn render_font_sets(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .mt(px(12.))
            .pt(px(10.))
            .border_t_1()
            .border_color(palette::border())
            .child(
                div()
                    .font_family(typography::mono())
                    .text_size(px(9.5))
                    .text_color(palette::muted())
                    .child("TYPEFACE"),
            )
            .child(
                div().flex().gap(px(4.)).mt(px(8.)).children(
                    catalog::font_sets()
                        .iter()
                        .enumerate()
                        .map(|(index, font_set)| {
                            let selected = index == self.selected_font;
                            div()
                                .id(("theme-font", index))
                                .flex_1()
                                .p(px(8.))
                                .rounded(px(8.))
                                .border_1()
                                .border_color(if selected {
                                    palette::accent()
                                } else {
                                    palette::border()
                                })
                                .bg(if selected {
                                    palette::raise()
                                } else {
                                    palette::transparent()
                                })
                                .cursor_pointer()
                                .hover(|style| style.border_color(palette::accent()))
                                .on_click(cx.listener(move |section, _event, _window, cx| {
                                    section.selected_font = index;
                                    cx.notify();
                                }))
                                .child(
                                    div()
                                        .font_family(font_set.ui_family)
                                        .text_size(px(12.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(if selected {
                                            palette::text_bright()
                                        } else {
                                            palette::text()
                                        })
                                        .child(font_set.label),
                                )
                                .child(
                                    div()
                                        .font_family(font_set.mono_family)
                                        .text_size(px(9.5))
                                        .text_color(palette::muted())
                                        .mt(px(2.))
                                        .child(font_set.sample),
                                )
                        }),
                ),
            )
    }

    fn render_accent_slots(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap(px(8.))
            .mt(px(12.))
            .pt(px(10.))
            .border_t_1()
            .border_color(palette::border())
            .child(
                div()
                    .text_size(px(11.5))
                    .text_color(palette::subtle())
                    .whitespace_nowrap()
                    .child("Accent slot"),
            )
            .child(div().ml_auto().flex().gap(px(4.)).children(
                ACCENT_SLOTS.iter().enumerate().map(|(index, &slot)| {
                    let selected = index == self.selected_accent;
                    div()
                        .id(("theme-accent", index))
                        .size(px(20.))
                        .rounded(px(6.))
                        .border_2()
                        .border_color(if selected {
                            palette::text_bright()
                        } else {
                            palette::transparent()
                        })
                        .bg(palette::slot(slot))
                        .cursor_pointer()
                        .on_click(cx.listener(move |section, _event, _window, cx| {
                            section.selected_accent = index;
                            cx.notify();
                        }))
                }),
            ))
    }
}

fn stripe(color: Rgba) -> gpui::Div {
    div().w(px(5.)).h(px(12.)).bg(color)
}

impl Render for ThemeSection {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let on_open_change = {
            let entity = cx.entity().downgrade();
            move |open: bool, _details: &mut _, _window: &mut Window, cx: &mut gpui::App| {
                entity
                    .update(cx, |section: &mut Self, cx| {
                        section.open = open;
                        section.set_popup_extent(if open { POPUP_EXTENT } else { 0.0 }, cx);
                        cx.notify();
                    })
                    .ok();
            }
        };

        let popover = PopoverRoot::<()>::new()
            .id("theme")
            .handle(self.popover.clone())
            .on_open_change(on_open_change)
            .child(
                PopoverTrigger::new()
                    .id("theme-trigger")
                    .aria_label("Theme")
                    .flex()
                    .items_center()
                    .gap(px(2.))
                    .h(px(22.))
                    .px(px(7.))
                    .rounded(px(6.))
                    .style_with_state(|state, trigger| {
                        if state.open {
                            trigger.bg(palette::raise())
                        } else {
                            trigger
                                .bg(palette::transparent())
                                .hover(|style| style.bg(palette::raise()))
                        }
                    })
                    .child(stripe(palette::red()).rounded_l(px(2.)))
                    .child(stripe(palette::yellow()))
                    .child(stripe(palette::green()))
                    .child(stripe(palette::blue()).rounded_r(px(2.))),
            )
            .child(
                PopoverPortal::new().child(
                    PopoverPositioner::new()
                        .side(PopoverSide::Bottom)
                        .align(PopoverAlign::End)
                        .side_offset(px(8.))
                        .collision_padding(px(0.))
                        .child(
                            PopoverPopup::new()
                                .id("theme-popup")
                                .w(px(344.))
                                .p(px(12.))
                                .rounded(px(12.))
                                .bg(palette::bar())
                                .border_1()
                                .border_color(palette::border())
                                .shadow_lg()
                                .child_any(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .font_family(typography::mono())
                                                .text_size(px(9.5))
                                                .text_color(palette::muted())
                                                .child("THEME"),
                                        )
                                        .child(self.render_appearance_toggle(cx))
                                        .into_any_element(),
                                )
                                .child_any(self.render_theme_list(cx).into_any_element())
                                .child_any(self.render_font_sets(cx).into_any_element())
                                .child_any(self.render_accent_slots(cx).into_any_element()),
                        ),
                ),
            );

        div()
            .id("theme-hover")
            .on_hover(cx.listener(|section, hovered: &bool, _window, cx| {
                if *hovered {
                    section.set_popup_extent(POPUP_EXTENT, cx);
                } else if !section.open {
                    section.set_popup_extent(0.0, cx);
                }
            }))
            .child(popover)
    }
}
