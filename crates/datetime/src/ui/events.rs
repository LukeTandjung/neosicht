use chrono::{Local, TimeZone as _};
use gpui::{Context, Render, Task, Window, div, prelude::*, px};
use theme::core::{palette, typography};

use crate::core::event::CalendarEvent;
use crate::ports::calendar::CalendarReadError;

pub struct UpcomingEventsSection {
    events: Vec<CalendarEvent>,
    error: Option<CalendarReadError>,
    observer: Option<Task<()>>,
}

impl UpcomingEventsSection {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            error: None,
            observer: None,
        }
    }

    pub fn own_observer(&mut self, observer: Task<()>) {
        self.observer = Some(observer);
    }

    pub fn apply(
        &mut self,
        loaded: Result<Vec<CalendarEvent>, CalendarReadError>,
        cx: &mut Context<Self>,
    ) {
        let changed = match loaded {
            Ok(events) => {
                let changed = self.events != events || self.error.is_some();
                self.events = events;
                self.error = None;
                changed
            }
            Err(error) => {
                let changed = self.error.as_ref() != Some(&error);
                self.error = Some(error);
                changed
            }
        };
        if changed {
            cx.notify();
        }
    }
}

impl Render for UpcomingEventsSection {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let message = match self.error {
            Some(CalendarReadError::PermissionDenied) => Some("Calendar access is required"),
            Some(_) => Some("Calendar unavailable"),
            None if self.events.is_empty() => Some("No events in the next 7 days"),
            None => None,
        };

        div()
            .mt(px(12.))
            .pt(px(10.))
            .border_t_1()
            .border_color(palette::border())
            .when_some(message, |list, message| {
                list.child(
                    div()
                        .font_family(typography::mono())
                        .text_size(px(9.5))
                        .text_color(palette::muted())
                        .child(message),
                )
            })
            .when(message.is_none(), |list| {
                list.flex()
                    .flex_col()
                    .gap(px(8.))
                    .children(self.events.iter().map(render_event).collect::<Vec<_>>())
            })
    }
}

fn render_event(event: &CalendarEvent) -> gpui::Div {
    let starts_at = Local
        .timestamp_opt(event.starts_at, 0)
        .single()
        .map(|time| {
            if event.all_day {
                time.format("%a · all day").to_string()
            } else {
                time.format("%a · %H:%M").to_string()
            }
        })
        .unwrap_or_else(|| "Unknown time".to_owned());

    div()
        .flex()
        .gap(px(8.))
        .items_center()
        .child(
            div()
                .flex_none()
                .w(px(3.))
                .h(px(24.))
                .rounded(px(2.))
                .bg(event
                    .calendar_color
                    .map(gpui::rgb)
                    .unwrap_or_else(palette::accent)),
        )
        .child(
            div()
                .min_w_0()
                .child(
                    div()
                        .text_size(px(11.5))
                        .text_color(palette::text_bright())
                        .truncate()
                        .child(event.title.clone()),
                )
                .child(
                    div()
                        .mt(px(1.))
                        .font_family(typography::mono())
                        .text_size(px(9.5))
                        .text_color(palette::subtle())
                        .truncate()
                        .child(format!("{starts_at} · {}", event.calendar_name)),
                ),
        )
}
