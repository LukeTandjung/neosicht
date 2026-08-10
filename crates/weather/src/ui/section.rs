//! Live weather presentation backed by the application weather service.

use chrono::{Datelike as _, NaiveDate};
use gpui::{AnyElement, Context, EventEmitter, FontWeight, Rgba, Task, div, prelude::*, px, rgba};
use theme::core::{palette, typography};

use crate::app::weather::WeatherLoadError;
use crate::core::forecast::{DailyForecast, WeatherReport, condition_name};

pub enum SectionEvent {
    Changed,
}

pub struct WeatherSection {
    report: Option<WeatherReport>,
    error: Option<WeatherLoadError>,
    observer: Option<Task<()>>,
}

impl EventEmitter<SectionEvent> for WeatherSection {}

impl WeatherSection {
    pub fn new() -> Self {
        Self {
            report: None,
            error: None,
            observer: None,
        }
    }

    pub fn own_observer(&mut self, observer: Task<()>) {
        self.observer = Some(observer);
    }

    pub fn apply(
        &mut self,
        loaded: Result<WeatherReport, WeatherLoadError>,
        cx: &mut Context<Self>,
    ) {
        let changed = match loaded {
            Ok(report) => {
                let changed = self.report.as_ref() != Some(&report) || self.error.is_some();
                self.report = Some(report);
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
            cx.emit(SectionEvent::Changed);
        }
    }

    pub fn bar_fragment(&self) -> AnyElement {
        let temperature = self
            .report
            .as_ref()
            .map(|report| format!("{:.0}°", report.temperature_celsius))
            .unwrap_or_else(|| "—°".to_owned());
        let tone = self
            .report
            .as_ref()
            .map(|report| condition_tone(report.weather_code))
            .unwrap_or_else(palette::muted);

        div()
            .flex()
            .items_center()
            .gap(px(6.))
            .child(
                div()
                    .size(px(9.))
                    .rounded_full()
                    .bg(tone)
                    .border_2()
                    .border_color(rgba(0xffffff12)),
            )
            .child(temperature)
            .into_any_element()
    }

    pub fn panel_column(&self) -> AnyElement {
        let Some(report) = &self.report else {
            let message = if self.error.is_some() {
                "Weather unavailable"
            } else {
                "Locating…"
            };
            return column_shell()
                .child(status_message(message))
                .into_any_element();
        };

        column_shell()
            .child(
                div()
                    .flex()
                    .items_start()
                    .justify_between()
                    .mt(px(8.))
                    .child(
                        div()
                            .child(
                                div()
                                    .text_size(px(12.5))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(palette::text_bright())
                                    .child(report.place_name.clone()),
                            )
                            .child(
                                div()
                                    .font_family(typography::mono())
                                    .text_size(px(10.5))
                                    .text_color(palette::subtle())
                                    .mt(px(2.))
                                    .child(condition_name(report.weather_code)),
                            )
                            .child(
                                div()
                                    .font_family(typography::mono())
                                    .text_size(px(10.5))
                                    .text_color(palette::subtle())
                                    .child(format!(
                                        "Feels {:.0}°",
                                        report.apparent_temperature_celsius
                                    )),
                            ),
                    )
                    .child(
                        div()
                            .font_family(typography::mono())
                            .text_size(px(28.))
                            .line_height(px(28.))
                            .text_color(condition_tone(report.weather_code))
                            .child(format!("{:.0}°", report.temperature_celsius)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .mt(px(12.))
                    .children(report.daily.iter().map(render_day)),
            )
            .into_any_element()
    }
}

fn column_shell() -> gpui::Div {
    div().flex_1().min_w_0().child(
        div()
            .font_family(typography::mono())
            .text_size(px(9.5))
            .text_color(palette::muted())
            .child("WEATHER"),
    )
}

fn status_message(message: &'static str) -> gpui::Div {
    div()
        .mt(px(10.))
        .font_family(typography::mono())
        .text_size(px(10.))
        .text_color(palette::muted())
        .child(message)
}

fn render_day(day: &DailyForecast) -> gpui::Div {
    let weekday = NaiveDate::parse_from_str(&day.date, "%Y-%m-%d")
        .map(|date| date.weekday().to_string()[..3].to_uppercase())
        .unwrap_or_else(|_| "---".to_owned());

    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .px(px(7.))
        .py(px(6.))
        .rounded(px(6.))
        .bg(palette::raise())
        .child(
            div()
                .w(px(26.))
                .font_family(typography::mono())
                .text_size(px(9.5))
                .text_color(palette::subtle())
                .child(weekday),
        )
        .child(
            div()
                .flex_1()
                .h(px(3.))
                .rounded(px(2.))
                .bg(condition_tone(day.weather_code))
                .opacity(0.55),
        )
        .child(
            div()
                .font_family(typography::mono())
                .text_size(px(10.5))
                .text_color(palette::text_bright())
                .child(format!(
                    "{:.0}° / {:.0}°",
                    day.high_celsius, day.low_celsius
                )),
        )
}

fn condition_tone(weather_code: u16) -> Rgba {
    match weather_code {
        0..=3 => palette::yellow(),
        45..=67 | 80..=82 => palette::cyan(),
        71..=77 | 85 | 86 => palette::blue(),
        95..=99 => palette::red(),
        _ => palette::muted(),
    }
}
