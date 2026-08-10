//! The bluetooth widget: a rune button opening the device popover with a
//! power switch and device list. Placeholder-only for now — devices are
//! static until a bluetooth provider feeds them.

use base_gpui::popover::{
    PopoverAlign, PopoverHandle, PopoverPopup, PopoverPortal, PopoverPositioner, PopoverRoot,
    PopoverSide, PopoverTrigger, create_popover_handle,
};
use base_gpui::switch::{SwitchRoot, SwitchThumb};
use gpui::{Context, EventEmitter, Window, div, prelude::*, px, svg};
use theme::core::{palette, typography};

/// Vertical room, in pixels, the shell must clear below the bar row while the
/// bluetooth popover is open.
pub const POPUP_EXTENT: f64 = 180.0;

pub const BLUETOOTH_ICON: &str = "bluetooth/bluetooth.svg";

pub const ASSETS: [(&str, &[u8]); 1] = [(BLUETOOTH_ICON, include_bytes!("icons/bluetooth.svg"))];

struct Device {
    name: &'static str,
    meta: &'static str,
    connected: bool,
}

const DEVICES: [Device; 3] = [
    Device {
        name: "AirPods Pro",
        meta: "84%",
        connected: true,
    },
    Device {
        name: "MX Master 3S",
        meta: "connected",
        connected: true,
    },
    Device {
        name: "HHKB Pro",
        meta: "paired",
        connected: false,
    },
];

pub enum SectionEvent {
    /// The section needs `extent` pixels of window below the bar row
    /// (0 = collapsed back to the bare bar).
    PopupExtentChanged { extent: f64 },
}

pub struct BluetoothSection {
    enabled: bool,
    open: bool,
    popup_extent: f64,
    popover: PopoverHandle<()>,
}

impl EventEmitter<SectionEvent> for BluetoothSection {}

impl BluetoothSection {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            enabled: true,
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

    fn render_switch(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let on_checked_change = {
            let entity = cx.entity().downgrade();
            move |checked: bool, _details: &mut _, _window: &mut Window, cx: &mut gpui::App| {
                entity
                    .update(cx, |section: &mut Self, cx| {
                        section.enabled = checked;
                        cx.notify();
                    })
                    .ok();
            }
        };

        SwitchRoot::new()
            .id("bluetooth-switch")
            .aria_label("Bluetooth")
            .checked(Some(self.enabled))
            .on_checked_change(on_checked_change)
            .w(px(34.))
            .h(px(19.))
            .p(px(2.))
            .rounded_full()
            .flex()
            .style_with_state(|state, root| {
                let root = if state.checked {
                    root.bg(palette::accent())
                } else {
                    root.bg(palette::raise())
                };
                root.justify_start()
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
            )
    }

    fn render_devices(&self) -> impl IntoElement {
        let enabled = self.enabled;
        div()
            .flex()
            .flex_col()
            .gap(px(2.))
            .mt(px(8.))
            .children(DEVICES.iter().map(move |device| {
                let live = enabled && device.connected;
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .p(px(8.))
                    .rounded(px(6.))
                    .text_size(px(12.))
                    .text_color(if !enabled {
                        palette::muted()
                    } else if device.connected {
                        palette::text()
                    } else {
                        palette::subtle()
                    })
                    .hover(|style| style.bg(palette::raise()))
                    .child(div().size(px(6.)).rounded_full().bg(if live {
                        palette::green()
                    } else {
                        palette::muted()
                    }))
                    .child(device.name)
                    .child(
                        div()
                            .ml_auto()
                            .font_family(typography::mono())
                            .text_size(px(10.))
                            .text_color(palette::muted())
                            .child(device.meta),
                    )
            }))
    }
}

impl Render for BluetoothSection {
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

        let popover =
            PopoverRoot::<()>::new()
                .id("bluetooth")
                .handle(self.popover.clone())
                .on_open_change(on_open_change)
                .child(
                    PopoverTrigger::new()
                        .id("bluetooth-trigger")
                        .aria_label("Bluetooth")
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
                        .child(svg().path(BLUETOOTH_ICON).size(px(15.)).text_color(
                            if self.enabled {
                                palette::blue()
                            } else {
                                palette::muted()
                            },
                        )),
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
                                    .id("bluetooth-popup")
                                    .w(px(248.))
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
                                                    .child("BLUETOOTH"),
                                            )
                                            .child(self.render_switch(cx))
                                            .into_any_element(),
                                    )
                                    .child_any(self.render_devices().into_any_element()),
                            ),
                    ),
                );

        div()
            .id("bluetooth-hover")
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
