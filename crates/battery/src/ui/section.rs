//! Live battery gauge backed by the application battery service.

use gpui::{Context, Render, Task, Window, div, prelude::*, px, relative};
use theme::core::{palette, typography};

use crate::core::status::BatteryStatus;
use crate::ports::power::BatteryReadError;

pub struct BatterySection {
    status: Option<BatteryStatus>,
    observer: Option<Task<()>>,
}

impl BatterySection {
    pub fn new() -> Self {
        Self {
            status: None,
            observer: None,
        }
    }

    pub(crate) fn own_observer(&mut self, observer: Task<()>) {
        self.observer = Some(observer);
    }

    pub(crate) fn apply(
        &mut self,
        observed: Result<Option<BatteryStatus>, BatteryReadError>,
        cx: &mut Context<Self>,
    ) {
        if let Ok(status) = observed
            && self.status != status
        {
            self.status = status;
            cx.notify();
        }
    }
}

impl Render for BatterySection {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let percentage = self.status.map(BatteryStatus::percentage);
        let charge = percentage.unwrap_or(0) as f32 / 100.0;
        let fill = match self.status {
            Some(status) if status.is_charging() => palette::green(),
            Some(status) if status.percentage() <= 20 => palette::red(),
            Some(_) => palette::accent(),
            None => palette::muted(),
        };

        div()
            .h(px(22.))
            .px(px(8.))
            .flex()
            .items_center()
            .gap(px(4.))
            .child(
                div()
                    .relative()
                    .w(px(22.))
                    .h(px(11.))
                    .rounded(px(4.))
                    .border_1()
                    .border_color(palette::subtle())
                    .p(px(1.5))
                    .child(div().w(relative(charge)).h_full().rounded(px(1.5)).bg(fill))
                    .child(
                        div()
                            .absolute()
                            .top(px(2.5))
                            .right(px(-3.5))
                            .w(px(2.))
                            .h(px(4.))
                            .rounded_r(px(2.))
                            .bg(palette::subtle()),
                    ),
            )
            .child(
                div()
                    .font_family(typography::mono())
                    .text_size(px(10.5))
                    .text_color(palette::text())
                    .child(
                        percentage
                            .map(|value| format!("{value}%"))
                            .unwrap_or_else(|| "—".to_owned()),
                    ),
            )
    }
}
