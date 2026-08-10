//! Live local clock and current-month calendar.

use chrono::{Datelike, NaiveDate};
use gpui::{Context, Entity, Render, Task, Window, div, prelude::*, px};
use theme::core::{palette, typography};

use crate::app::clock::ClockSnapshot;
use crate::core::calendar::month_grid;
use crate::ui::events::UpcomingEventsSection;

/// A live local clock, refreshed once per second without blocking GPUI.
pub struct ClockSection {
    snapshot: ClockSnapshot,
    observer: Option<Task<()>>,
}

impl ClockSection {
    pub fn new(snapshot: ClockSnapshot) -> Self {
        Self {
            snapshot,
            observer: None,
        }
    }

    pub fn own_observer(&mut self, observer: Task<()>) {
        self.observer = Some(observer);
    }

    pub fn local_date(&self) -> NaiveDate {
        self.snapshot.local_date
    }

    pub fn apply(&mut self, snapshot: ClockSnapshot, cx: &mut Context<Self>) {
        if self.snapshot != snapshot {
            self.snapshot = snapshot;
            cx.notify();
        }
    }
}

impl Render for ClockSection {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .text_color(palette::text_bright())
            .child(self.snapshot.display.clone())
    }
}

/// Current local month calendar. It is rebuilt from the current date whenever
/// the island renders, so month and year rollover require no special state.
pub fn panel_column(
    local_date: NaiveDate,
    events: Entity<UpcomingEventsSection>,
) -> impl IntoElement {
    let days = month_grid(local_date).unwrap_or_default();
    let heading = local_date.format("%B %Y").to_string().to_uppercase();
    let week = format!("W{:02}", local_date.iso_week().week());

    div()
        .flex_1()
        .min_w_0()
        .border_l_1()
        .border_r_1()
        .border_color(palette::border())
        .px(px(15.))
        .child(
            div()
                .flex()
                .items_baseline()
                .justify_between()
                .child(
                    div()
                        .font_family(typography::mono())
                        .text_size(px(9.5))
                        .text_color(palette::muted())
                        .child(heading),
                )
                .child(
                    div()
                        .font_family(typography::mono())
                        .text_size(px(9.))
                        .text_color(palette::muted())
                        .child(week),
                ),
        )
        .child(
            div().mt(px(8.)).child(
                div()
                    .flex()
                    .children(["S", "M", "T", "W", "T", "F", "S"].map(|name| {
                        div()
                            .flex_1()
                            .text_center()
                            .py(px(2.))
                            .font_family(typography::mono())
                            .text_size(px(8.5))
                            .text_color(palette::muted())
                            .child(name)
                    })),
            ),
        )
        .children((0..6).map(move |week_index| {
            div().flex().children((0..7).map({
                let days = days.clone();
                move |weekday| {
                    let day = days[week_index * 7 + weekday];
                    div()
                        .flex_1()
                        .h(px(21.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(6.))
                        .font_family(typography::mono())
                        .text_size(px(10.))
                        .when(day.today, |cell| cell.bg(palette::accent()))
                        .text_color(if day.today {
                            palette::ink()
                        } else if day.in_month {
                            palette::text()
                        } else {
                            palette::muted()
                        })
                        .child(day.number.to_string())
                }
            }))
        }))
        .child(events)
}

// A fixed-capacity month grid avoids heap allocation while retaining a named
// type at the calendar boundary.
