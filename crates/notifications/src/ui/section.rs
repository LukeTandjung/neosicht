//! The notification widget: a bell button with an unread badge opening the
//! notification center popover. Placeholder-only for now — the cards are
//! static samples until a notification provider feeds them.

use base_gpui::popover::{
    PopoverAlign, PopoverHandle, PopoverPopup, PopoverPortal, PopoverPositioner, PopoverRoot,
    PopoverSide, PopoverTrigger, create_popover_handle,
};
use base_gpui::switch::{SwitchRoot, SwitchThumb};
use gpui::{Context, EventEmitter, FontWeight, Rgba, Window, div, prelude::*, px, svg};
use theme::core::{palette, typography};

/// Vertical room, in pixels, the shell must clear below the bar row while the
/// notification popover is open.
pub const POPUP_EXTENT: f64 = 320.0;

pub const BELL_ICON: &str = "notifications/bell.svg";

pub const ASSETS: [(&str, &[u8]); 1] = [(BELL_ICON, include_bytes!("icons/bell.svg"))];

#[derive(Clone)]
struct Note {
    initial: &'static str,
    tone: Rgba,
    title: &'static str,
    body: &'static str,
    ago: &'static str,
}

fn sample_notes() -> Vec<Note> {
    vec![
        Note {
            initial: "Z",
            tone: palette::blue(),
            title: "Zed",
            body: "bar.rs — 2 unresolved comments on your PR",
            ago: "2m",
        },
        Note {
            initial: "C",
            tone: palette::green(),
            title: "Chat",
            body: "#ricing — “post the gruvbox variant?”",
            ago: "14m",
        },
        Note {
            initial: "M",
            tone: palette::orange(),
            title: "Mail",
            body: "Homebrew: aerospace 0.19 released",
            ago: "1h",
        },
    ]
}

pub enum SectionEvent {
    /// The section needs `extent` pixels of window below the bar row
    /// (0 = collapsed back to the bare bar).
    PopupExtentChanged { extent: f64 },
}

pub struct NotificationsSection {
    notes: Vec<Note>,
    do_not_disturb: bool,
    open: bool,
    popup_extent: f64,
    popover: PopoverHandle<()>,
}

impl EventEmitter<SectionEvent> for NotificationsSection {}

impl NotificationsSection {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            notes: sample_notes(),
            do_not_disturb: false,
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

    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            .px(px(4.))
            .pt(px(2.))
            .pb(px(9.))
            .child(
                div()
                    .font_family(typography::mono())
                    .text_size(px(9.5))
                    .text_color(palette::muted())
                    .child("NOTIFICATION CENTER"),
            )
            .child(
                div()
                    .id("notifications-clear")
                    .font_family(typography::mono())
                    .text_size(px(9.5))
                    .text_color(palette::accent())
                    .cursor_pointer()
                    .on_click(cx.listener(|section, _event, _window, cx| {
                        section.notes.clear();
                        cx.notify();
                    }))
                    .child("CLEAR ALL"),
            )
    }

    fn render_notes(&self) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .children(self.notes.iter().map(|note| {
                div()
                    .flex()
                    .gap(px(10.))
                    .p(px(10.))
                    .rounded(px(8.))
                    .bg(palette::raise())
                    .border_1()
                    .border_color(palette::border())
                    .child(
                        div()
                            .flex_none()
                            .size(px(24.))
                            .rounded(px(6.))
                            .bg(note.tone)
                            .flex()
                            .items_center()
                            .justify_center()
                            .font_family(typography::mono())
                            .text_size(px(10.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(palette::ink())
                            .child(note.initial),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .flex()
                                    .items_baseline()
                                    .gap(px(8.))
                                    .child(
                                        div()
                                            .text_size(px(11.5))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(palette::text_bright())
                                            .child(note.title),
                                    )
                                    .child(
                                        div()
                                            .ml_auto()
                                            .flex_none()
                                            .font_family(typography::mono())
                                            .text_size(px(9.5))
                                            .text_color(palette::muted())
                                            .child(note.ago),
                                    ),
                            )
                            .child(
                                div()
                                    .text_size(px(11.5))
                                    .text_color(palette::subtle())
                                    .mt(px(3.))
                                    .line_height(px(15.5))
                                    .child(note.body),
                            ),
                    )
            }))
    }

    fn render_do_not_disturb(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let on_checked_change = {
            let entity = cx.entity().downgrade();
            move |checked: bool, _details: &mut _, _window: &mut Window, cx: &mut gpui::App| {
                entity
                    .update(cx, |section: &mut Self, cx| {
                        section.do_not_disturb = checked;
                        cx.notify();
                    })
                    .ok();
            }
        };

        div()
            .flex()
            .items_center()
            .gap(px(8.))
            .mt(px(10.))
            .pt(px(10.))
            .border_t_1()
            .border_color(palette::border())
            .child(
                div()
                    .text_size(px(11.5))
                    .text_color(palette::subtle())
                    .child("Do Not Disturb"),
            )
            .child(
                SwitchRoot::new()
                    .id("notifications-dnd")
                    .aria_label("Do Not Disturb")
                    .checked(Some(self.do_not_disturb))
                    .on_checked_change(on_checked_change)
                    .ml_auto()
                    .w(px(34.))
                    .h(px(19.))
                    .p(px(2.))
                    .rounded_full()
                    .flex()
                    .style_with_state(|state, root| {
                        if state.checked {
                            root.bg(palette::accent())
                        } else {
                            root.bg(palette::raise())
                        }
                    })
                    .child(
                        SwitchThumb::new()
                            .size(px(15.))
                            .rounded_full()
                            .bg(palette::bg())
                            .style_with_state(|state, thumb| {
                                if state.root.checked {
                                    thumb.ml(px(15.))
                                } else {
                                    thumb
                                }
                            }),
                    ),
            )
    }
}

impl Render for NotificationsSection {
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

        let badge_visible = !self.do_not_disturb && !self.notes.is_empty();

        let popover = PopoverRoot::<()>::new()
            .id("notifications")
            .handle(self.popover.clone())
            .on_open_change(on_open_change)
            .child(
                PopoverTrigger::new()
                    .id("notifications-trigger")
                    .aria_label("Notifications")
                    .relative()
                    .flex()
                    .items_center()
                    .h(px(22.))
                    .px(px(8.))
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
                    .child(
                        svg()
                            .path(BELL_ICON)
                            .size(px(15.))
                            .text_color(palette::text()),
                    )
                    .when(badge_visible, |trigger| {
                        trigger.child(
                            div()
                                .absolute()
                                .top(px(2.))
                                .right(px(3.5))
                                .size(px(9.))
                                .rounded_full()
                                .bg(palette::bar())
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(div().size(px(6.)).rounded_full().bg(palette::red())),
                        )
                    }),
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
                                .id("notifications-popup")
                                .w(px(312.))
                                .p(px(10.))
                                .rounded(px(12.))
                                .bg(palette::bar())
                                .border_1()
                                .border_color(palette::border())
                                .shadow_lg()
                                .child_any(self.render_header(cx).into_any_element())
                                .child_any(self.render_notes().into_any_element())
                                .child_any(self.render_do_not_disturb(cx).into_any_element()),
                        ),
                ),
            );

        div()
            .id("notifications-hover")
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
