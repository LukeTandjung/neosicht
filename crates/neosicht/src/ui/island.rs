//! The weather, now-playing, and live-time island.

use base_gpui::popover::{
    PopoverAlign, PopoverHandle, PopoverPopup, PopoverPortal, PopoverPositioner, PopoverRoot,
    PopoverSide, PopoverTrigger, create_popover_handle,
};
use datetime::ui::events::UpcomingEventsSection;
use datetime::ui::section::ClockSection;
use gpui::{Context, Entity, EventEmitter, Window, div, prelude::*, px};
use music::ui::section::MusicSection;
use theme::core::{palette, typography};
use weather::ui::section::WeatherSection;

pub const POPUP_EXTENT: f64 = 380.0;

pub enum IslandEvent {
    PopupExtentChanged { extent: f64 },
}

pub struct IslandSection {
    open: bool,
    popup_extent: f64,
    popover: PopoverHandle<()>,
    clock: Entity<ClockSection>,
    events: Entity<UpcomingEventsSection>,
    weather: Entity<WeatherSection>,
    music: Entity<MusicSection>,
}

impl EventEmitter<IslandEvent> for IslandSection {}

impl IslandSection {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let weather = weather::impls::section::macos_section(cx);
        cx.subscribe(&weather, |_island, _section, _event, cx| cx.notify())
            .detach();
        let music = music::impls::section::applescript_section(cx);
        cx.subscribe(&music, |_island, _section, _event, cx| cx.notify())
            .detach();

        Self {
            open: false,
            popup_extent: 0.0,
            popover: create_popover_handle(),
            clock: datetime::impls::clock::system_clock_section(cx),
            events: datetime::impls::events::eventkit_section(cx),
            weather,
            music,
        }
    }

    fn set_popup_extent(&mut self, extent: f64, cx: &mut Context<Self>) {
        if self.popup_extent == extent {
            return;
        }
        self.popup_extent = extent;
        cx.emit(IslandEvent::PopupExtentChanged { extent });
    }
}

fn divider() -> gpui::Div {
    div().w(px(1.)).h(px(12.)).bg(palette::border())
}

impl Render for IslandSection {
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
            .id("island")
            .handle(self.popover.clone())
            .on_open_change(on_open_change)
            .child(
                PopoverTrigger::new()
                    .id("island-trigger")
                    .aria_label("Weather, music, and calendar")
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .h(px(22.))
                    .px(px(8.))
                    .rounded(px(8.))
                    .border_1()
                    .font_family(typography::mono())
                    .text_size(px(11.))
                    .text_color(palette::text())
                    .whitespace_nowrap()
                    .style_with_state(|state, trigger| {
                        if state.open {
                            trigger.bg(palette::raise()).border_color(palette::accent())
                        } else {
                            trigger
                                .bg(palette::transparent())
                                .border_color(palette::border())
                                .hover(|style| style.bg(palette::raise()))
                        }
                    })
                    .child(self.weather.read(cx).bar_fragment())
                    .child(divider())
                    .child(self.music.read(cx).bar_fragment())
                    .child(divider())
                    .child(self.clock.clone()),
            )
            .child(
                PopoverPortal::new().child(
                    PopoverPositioner::new()
                        .side(PopoverSide::Bottom)
                        .align(PopoverAlign::End)
                        // The trigger sits at the bar row's 12px padding; push the
                        // popup right so its edge lines up with the bar's edge.
                        .align_offset(px(12.))
                        .side_offset(px(8.))
                        .collision_padding(px(0.))
                        .child(
                            PopoverPopup::new()
                                .id("island-popup")
                                .w(px(604.))
                                .p(px(16.))
                                .rounded(px(12.))
                                .bg(palette::bar())
                                .border_1()
                                .border_color(palette::border())
                                .shadow_lg()
                                .flex()
                                .items_start()
                                .gap(px(16.))
                                .child_any(self.weather.read(cx).panel_column())
                                .child_any(
                                    datetime::ui::section::panel_column(
                                        self.clock.read(cx).local_date(),
                                        self.events.clone(),
                                    )
                                    .into_any_element(),
                                )
                                .child_any(
                                    self.music.read(cx).panel_column(self.music.downgrade()),
                                ),
                        ),
                ),
            );

        div()
            .id("island-hover")
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
