use std::sync::{Arc, Mutex};

use base_gpui::button::ButtonRoot;
use base_gpui::checkbox::{CheckboxIndicator, CheckboxRoot};
use base_gpui::dialog::{
    DialogBackdrop, DialogHandle, DialogPopup, DialogPortal, DialogRoot, DialogViewport,
    create_dialog_handle,
};
use base_gpui::input::Input;
use base_gpui::popover::{
    PopoverAlign, PopoverHandle, PopoverPopup, PopoverPortal, PopoverPositioner, PopoverRoot,
    PopoverSide, PopoverTrigger, create_popover_handle,
};
use base_gpui::switch::{SwitchRoot, SwitchThumb};
use gpui::{Context, EventEmitter, FontWeight, WeakEntity, Window, div, prelude::*, px, svg};
use theme::core::{palette, typography};

use crate::app::wifi::WifiService;
use crate::core::network::WifiSnapshot;

/// Vertical room, in pixels, the shell must clear below the bar row while the
/// wi-fi popover is open.
pub const POPUP_EXTENT: f64 = 210.0;

/// Extra room while the WPA2 join dialog is open below the bar.
pub const JOIN_EXTENT: f64 = 460.0;

pub const WIFI_ICON: &str = "wifi/wifi.svg";
pub const LOCK_ICON: &str = "wifi/lock.svg";

pub const ASSETS: [(&str, &[u8]); 2] = [
    (WIFI_ICON, include_bytes!("icons/wifi.svg")),
    (LOCK_ICON, include_bytes!("icons/lock.svg")),
];

pub enum SectionEvent {
    /// The section needs `extent` pixels of window below the bar row
    /// (0 = collapsed back to the bare bar).
    PopupExtentChanged { extent: f64 },
}

pub struct WifiSection {
    snapshot: WifiSnapshot,
    service: Arc<WifiService>,
    remember: bool,
    password: Arc<Mutex<String>>,
    open: bool,
    join_open: bool,
    popup_extent: f64,
    popover: PopoverHandle<()>,
    join_dialog: DialogHandle<usize>,
}

impl EventEmitter<SectionEvent> for WifiSection {}

impl WifiSection {
    pub fn new(service: Arc<WifiService>, _cx: &mut Context<Self>) -> Self {
        Self {
            snapshot: WifiSnapshot::default(),
            service,
            remember: true,
            password: Arc::new(Mutex::new(String::new())),
            open: false,
            join_open: false,
            popup_extent: 0.0,
            popover: create_popover_handle(),
            join_dialog: create_dialog_handle(),
        }
    }

    pub fn apply(
        &mut self,
        snapshot: Result<WifiSnapshot, crate::ports::wifi::WifiError>,
        cx: &mut Context<Self>,
    ) {
        if let Ok(snapshot) = snapshot
            && self.snapshot != snapshot
        {
            self.snapshot = snapshot;
            cx.notify();
        }
    }

    fn set_popup_extent(&mut self, extent: f64, cx: &mut Context<Self>) {
        if self.popup_extent == extent {
            return;
        }
        self.popup_extent = extent;
        cx.emit(SectionEvent::PopupExtentChanged { extent });
    }

    fn resting_extent(&self) -> f64 {
        if self.join_open {
            JOIN_EXTENT
        } else if self.open {
            POPUP_EXTENT
        } else {
            0.0
        }
    }

    fn render_switch(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let on_checked_change = {
            let entity = cx.entity().downgrade();
            move |checked: bool, _details: &mut _, _window: &mut Window, cx: &mut gpui::App| {
                entity
                    .update(cx, |section: &mut Self, cx| {
                        if section.service.set_enabled(checked).is_ok() {
                            section.snapshot.enabled = checked;
                            cx.notify();
                        }
                    })
                    .ok();
            }
        };

        SwitchRoot::new()
            .id("wifi-switch")
            .aria_label("Wi-Fi")
            .checked(Some(self.snapshot.enabled))
            .on_checked_change(on_checked_change)
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
            )
    }

    fn render_networks(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let enabled = self.snapshot.enabled;
        let connected = self.snapshot.connected_ssid.clone();
        div()
            .flex()
            .flex_col()
            .gap(px(2.))
            .mt(px(8.))
            .when(self.snapshot.networks.is_empty(), |list| {
                list.child(
                    div()
                        .p(px(8.))
                        .text_size(px(11.5))
                        .text_color(palette::muted())
                        .child(if enabled {
                            "No networks found"
                        } else {
                            "Wi-Fi is off"
                        }),
                )
            })
            .children(
                self.snapshot
                    .networks
                    .iter()
                    .enumerate()
                    .map(|(index, network)| {
                        let current = connected.as_deref() == Some(network.ssid.as_str());
                        let ssid = network.ssid.clone();
                        let secure = network.secure;
                        let bars = match network.signal {
                            -55.. => "▮▮▮",
                            -70..=-56 => "▮▮▯",
                            _ => "▮▯▯",
                        };
                        div()
                            .id(("wifi-network", index))
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .p(px(8.))
                            .rounded(px(6.))
                            .text_size(px(12.))
                            .text_color(if enabled {
                                palette::text()
                            } else {
                                palette::muted()
                            })
                            .bg(if current {
                                palette::raise()
                            } else {
                                palette::transparent()
                            })
                            .cursor_pointer()
                            .hover(|style| style.bg(palette::raise()))
                            .on_click(cx.listener(move |section, _event, window, cx| {
                                if !section.snapshot.enabled || current {
                                    return;
                                }
                                if section.service.join(&ssid, None).is_ok() {
                                    section.snapshot.connected_ssid = Some(ssid.clone());
                                    cx.notify();
                                } else if secure {
                                    section.password.lock().expect("password lock").clear();
                                    let handle = section.join_dialog.clone();
                                    window.defer(cx, move |window, cx| {
                                        handle.open_with_payload(index, window, cx);
                                    });
                                }
                            }))
                            .child(div().size(px(6.)).rounded_full().bg(if current {
                                palette::green()
                            } else {
                                palette::muted()
                            }))
                            .child(network.ssid.clone())
                            .child(
                                div()
                                    .ml_auto()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    .when(network.secure, |meta| {
                                        meta.child(
                                            svg()
                                                .path(LOCK_ICON)
                                                .size(px(11.))
                                                .text_color(palette::muted()),
                                        )
                                    })
                                    .child(
                                        div()
                                            .font_family(typography::mono())
                                            .text_size(px(10.))
                                            .text_color(palette::muted())
                                            .child(bars),
                                    ),
                            )
                    }),
            )
    }

    fn render_join_dialog(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity().downgrade();
        let on_open_change = {
            let entity = entity.clone();
            move |open: bool, _details: &mut _, _window: &mut Window, cx: &mut gpui::App| {
                entity
                    .update(cx, |section: &mut Self, cx| {
                        section.join_open = open;
                        section.set_popup_extent(section.resting_extent(), cx);
                        cx.notify();
                    })
                    .ok();
            }
        };

        // Non-modal on purpose: the shell window never becomes key, so Escape
        // can't reach the dialog — a press outside the popup must dismiss it
        // or a missed Cancel click leaves it stuck over the desktop.
        DialogRoot::<usize>::new()
            .id("wifi-join")
            .handle(self.join_dialog.clone())
            .modal(false)
            .on_open_change(on_open_change)
            .child(
                DialogPortal::new()
                    .child(
                        DialogBackdrop::new()
                            .absolute()
                            .inset_0()
                            .bg(palette::scrim()),
                    )
                    .child(
                        DialogViewport::new()
                            .absolute()
                            .inset_0()
                            .flex()
                            .items_start()
                            .justify_center()
                            // ~16vh on the design's canvas; the window now
                            // spans the full screen, so this reads the same.
                            .pt(px(160.))
                            .child(
                                DialogPopup::new()
                                    .id("wifi-join-popup")
                                    .aria_label("Join Wi-Fi network")
                                    .w(px(352.))
                                    .p(px(16.))
                                    .rounded(px(12.))
                                    .bg(palette::bar())
                                    .border_1()
                                    .border_color(palette::border())
                                    .shadow_lg()
                                    .payload_content(move |payload, _window, cx| {
                                        join_dialog_content(payload.copied(), &entity, cx)
                                    }),
                            ),
                    ),
            )
    }
}

fn join_dialog_content(
    network: Option<usize>,
    entity: &WeakEntity<WifiSection>,
    cx: &mut gpui::App,
) -> gpui::AnyElement {
    let index = network.unwrap_or(0);
    let name = entity
        .upgrade()
        .and_then(|section| {
            section
                .read(cx)
                .snapshot
                .networks
                .get(index)
                .map(|network| network.ssid.clone())
        })
        .unwrap_or_else(|| "Wi-Fi network".to_owned());
    let remember = entity
        .upgrade()
        .map(|section| section.read(cx).remember)
        .unwrap_or(true);

    let on_remember_change = {
        let entity = entity.clone();
        move |checked: bool, _details: &mut _, _window: &mut Window, cx: &mut gpui::App| {
            entity
                .update(cx, |section: &mut WifiSection, cx| {
                    section.remember = checked;
                    cx.notify();
                })
                .ok();
        }
    };

    let cancel = {
        let entity = entity.clone();
        move |_: &_, window: &mut Window, cx: &mut gpui::App| {
            if let Some(section) = entity.upgrade() {
                section.read(cx).join_dialog.clone().close(window, cx);
            }
        }
    };

    let confirm = {
        let entity = entity.clone();
        move |_: &_, window: &mut Window, cx: &mut gpui::App| {
            entity
                .update(cx, |section: &mut WifiSection, cx| {
                    let Some(network) = section.snapshot.networks.get(index) else {
                        return;
                    };
                    let ssid = network.ssid.clone();
                    let password = section.password.lock().expect("password lock").clone();
                    if section.service.join(&ssid, Some(&password)).is_ok() {
                        section.snapshot.connected_ssid = Some(ssid);
                        cx.notify();
                    }
                })
                .ok();
            if let Some(section) = entity.upgrade() {
                section.read(cx).join_dialog.clone().close(window, cx);
            }
        }
    };

    div()
        .child(
            div()
                .flex()
                .items_start()
                .gap(px(12.))
                .child(
                    svg()
                        .path(LOCK_ICON)
                        .size(px(17.))
                        .flex_none()
                        .mt(px(1.))
                        .text_color(palette::accent()),
                )
                .child(
                    // flex_1 + min_w_0 so the sentence wraps inside the
                    // dialog; a bare flex child sizes to its unwrapped text.
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_size(px(13.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(palette::text_bright())
                        .child(format!("The network “{name}” requires a WPA2 password.")),
                ),
        )
        .child(
            Input::new()
                .id("wifi-join-password")
                .on_value_change({
                    let password = entity
                        .upgrade()
                        .map(|section| section.read(cx).password.clone())
                        .unwrap_or_default();
                    move |value| *password.lock().expect("password lock") = value.to_string()
                })
                .name("password")
                .aria_label("Password")
                .placeholder("Password")
                .w_full()
                .mt(px(16.))
                .px(px(11.))
                .py(px(8.))
                .rounded(px(8.))
                .border_1()
                .border_color(palette::border())
                .bg(palette::raise())
                .text_size(px(13.))
                .text_color(palette::text_bright())
                .style_with_state(|state, input| {
                    if state.focused {
                        input.border_color(palette::accent())
                    } else {
                        input
                    }
                }),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .mt(px(12.))
                .child(
                    CheckboxRoot::new()
                        .id("wifi-join-remember")
                        .aria_label("Remember this network")
                        .checked(Some(remember))
                        .on_checked_change(on_remember_change)
                        .size(px(15.))
                        .rounded(px(4.))
                        .border_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .style_with_state(|state, root| {
                            if state.checked {
                                root.bg(palette::accent()).border_color(palette::accent())
                            } else {
                                root.bg(palette::transparent())
                                    .border_color(palette::border())
                            }
                        })
                        .child(
                            CheckboxIndicator::new().child(
                                div()
                                    .font_family(typography::mono())
                                    .text_size(px(9.))
                                    .text_color(palette::ink())
                                    .child("✓"),
                            ),
                        ),
                )
                .child(
                    div()
                        .text_size(px(11.5))
                        .text_color(palette::subtle())
                        .child("Remember this network"),
                ),
        )
        .child(
            div()
                .flex()
                .justify_end()
                .gap(px(8.))
                .mt(px(16.))
                .child(
                    ButtonRoot::new()
                        .id("wifi-join-cancel")
                        .aria_label("Cancel")
                        .px(px(16.))
                        .py(px(8.))
                        .rounded(px(8.))
                        .border_1()
                        .border_color(palette::border())
                        .text_size(px(12.5))
                        .text_color(palette::text())
                        .style_with_state(|_state, button| {
                            button.hover(|style| style.bg(palette::raise()))
                        })
                        .on_click(cancel)
                        .child("Cancel"),
                )
                .child(
                    ButtonRoot::new()
                        .id("wifi-join-confirm")
                        .aria_label("Join")
                        .px(px(16.))
                        .py(px(8.))
                        .rounded(px(8.))
                        .bg(palette::accent())
                        .text_size(px(12.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(palette::ink())
                        .on_click(confirm)
                        .child("Join"),
                ),
        )
        .into_any_element()
}

impl Render for WifiSection {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let on_open_change = {
            let entity = cx.entity().downgrade();
            move |open: bool, _details: &mut _, _window: &mut Window, cx: &mut gpui::App| {
                entity
                    .update(cx, |section: &mut Self, cx| {
                        section.open = open;
                        section.set_popup_extent(section.resting_extent(), cx);
                        cx.notify();
                    })
                    .ok();
            }
        };

        let popover = PopoverRoot::<()>::new()
            .id("wifi")
            .handle(self.popover.clone())
            .on_open_change(on_open_change)
            .child(
                PopoverTrigger::new()
                    .id("wifi-trigger")
                    .aria_label("Wi-Fi")
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
                    .child(svg().path(WIFI_ICON).size(px(16.)).text_color(
                        if self.snapshot.enabled {
                            palette::text()
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
                                .id("wifi-popup")
                                .w(px(264.))
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
                                                .child("WI-FI"),
                                        )
                                        .child(self.render_switch(cx))
                                        .into_any_element(),
                                )
                                .child_any(self.render_networks(cx).into_any_element()),
                        ),
                ),
            );

        div()
            .id("wifi-hover")
            .on_hover(cx.listener(|section, hovered: &bool, _window, cx| {
                if *hovered {
                    section.set_popup_extent(section.resting_extent().max(POPUP_EXTENT), cx);
                } else {
                    section.set_popup_extent(section.resting_extent(), cx);
                }
            }))
            .child(popover)
            .child(self.render_join_dialog(cx))
    }
}
