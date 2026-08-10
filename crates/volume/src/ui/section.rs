//! The volume widget: a speaker button opening the OUTPUT popover with a
//! level slider and output-device list. Placeholder-only for now — the level
//! and devices are local UI state until an audio provider feeds them.

use base_gpui::popover::{
    PopoverAlign, PopoverHandle, PopoverPopup, PopoverPortal, PopoverPositioner, PopoverRoot,
    PopoverSide, PopoverTrigger, create_popover_handle,
};
use base_gpui::slider::{
    SliderControl, SliderIndicator, SliderRoot, SliderThumb, SliderTrack, SliderValues,
};
use gpui::{Context, EventEmitter, Window, div, prelude::*, px, svg};
use theme::core::{palette, typography};

/// Vertical room, in pixels, the shell must clear below the bar row while the
/// volume popover is open.
pub const POPUP_EXTENT: f64 = 200.0;

pub const SPEAKER_ICON: &str = "volume/speaker.svg";

pub const ASSETS: [(&str, &[u8]); 1] = [(SPEAKER_ICON, include_bytes!("icons/speaker.svg"))];

const OUTPUTS: [&str; 3] = ["MacBook Pro Speakers", "AirPods Pro", "Studio Monitors"];

pub enum SectionEvent {
    /// The section needs `extent` pixels of window below the bar row
    /// (0 = collapsed back to the bare bar).
    PopupExtentChanged { extent: f64 },
}

pub struct VolumeSection {
    level: f64,
    selected_output: usize,
    open: bool,
    popup_extent: f64,
    popover: PopoverHandle<()>,
}

impl EventEmitter<SectionEvent> for VolumeSection {}

impl VolumeSection {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            level: 62.0,
            selected_output: 0,
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

    fn render_slider(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let on_value_change = {
            let entity = cx.entity().downgrade();
            move |values: SliderValues,
                  _details: &mut _,
                  _window: &mut Window,
                  cx: &mut gpui::App| {
                let level = match values {
                    SliderValues::Single(value) => value,
                    SliderValues::Range(ref range) => range.first().copied().unwrap_or(0.),
                };
                entity
                    .update(cx, |section: &mut Self, cx| {
                        section.level = level;
                        cx.notify();
                    })
                    .ok();
            }
        };

        SliderRoot::new()
            .id("volume-slider")
            .name("volume")
            .aria_label("Output volume")
            .min(0.)
            .max(100.)
            .step(1.)
            .value(SliderValues::Single(self.level))
            .on_value_change(on_value_change)
            .w_full()
            .mt(px(8.))
            .child(
                SliderControl::new()
                    .id("volume-slider-control")
                    .w_full()
                    .h(px(22.))
                    .child(
                        SliderTrack::new()
                            .id("volume-slider-track")
                            .w_full()
                            .mt(px(8.5))
                            .h(px(5.))
                            .rounded(px(3.))
                            .bg(palette::raise())
                            .child(
                                SliderIndicator::new()
                                    .id("volume-slider-indicator")
                                    .h(px(5.))
                                    .rounded(px(3.))
                                    .bg(palette::accent()),
                            ),
                    )
                    .child(
                        SliderThumb::new()
                            .id("volume-slider-thumb")
                            .aria_label("Output volume")
                            .mt(px(-9.))
                            .size(px(13.))
                            .rounded_full()
                            .bg(palette::text_bright())
                            .shadow_sm(),
                    ),
            )
    }

    fn render_outputs(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap(px(2.))
            .mt(px(8.))
            .children(OUTPUTS.iter().enumerate().map(|(index, name)| {
                let selected = index == self.selected_output;
                div()
                    .id(("volume-output", index))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .p(px(8.))
                    .rounded(px(6.))
                    .text_size(px(12.))
                    .text_color(palette::text())
                    .bg(if selected {
                        palette::raise()
                    } else {
                        palette::transparent()
                    })
                    .cursor_pointer()
                    .hover(|style| style.bg(palette::raise()))
                    .on_click(cx.listener(move |section, _event, _window, cx| {
                        section.selected_output = index;
                        cx.notify();
                    }))
                    .child(div().size(px(6.)).rounded_full().bg(if selected {
                        palette::accent()
                    } else {
                        palette::muted()
                    }))
                    .child(*name)
            }))
    }
}

impl Render for VolumeSection {
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
            .id("volume")
            .handle(self.popover.clone())
            .on_open_change(on_open_change)
            .child(
                PopoverTrigger::new()
                    .id("volume-trigger")
                    .aria_label("Volume")
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
                            .path(SPEAKER_ICON)
                            .size(px(16.))
                            .text_color(palette::text()),
                    ),
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
                                .id("volume-popup")
                                .w(px(252.))
                                .p(px(12.))
                                .rounded(px(12.))
                                .bg(palette::bar())
                                .border_1()
                                .border_color(palette::border())
                                .shadow_lg()
                                .child_any(
                                    div()
                                        .flex()
                                        .items_baseline()
                                        .justify_between()
                                        .child(
                                            div()
                                                .font_family(typography::mono())
                                                .text_size(px(9.5))
                                                .text_color(palette::muted())
                                                .child("OUTPUT"),
                                        )
                                        .child(
                                            div()
                                                .font_family(typography::mono())
                                                .text_size(px(10.5))
                                                .text_color(palette::subtle())
                                                .child(format!("{:.0}%", self.level)),
                                        )
                                        .into_any_element(),
                                )
                                .child_any(self.render_slider(cx).into_any_element())
                                .child_any(self.render_outputs(cx).into_any_element()),
                        ),
                ),
            );

        div()
            .id("volume-hover")
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
