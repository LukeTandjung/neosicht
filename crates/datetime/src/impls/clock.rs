use std::sync::Arc;
use std::thread;
use std::time::Duration;

use gpui::{App, AppContext, Entity};

use crate::adapters::system_clock::SystemClock;
use crate::app::clock::ClockService;
use crate::ui::section::ClockSection;

const REFRESH_INTERVAL: Duration = Duration::from_secs(1);

pub fn system_clock_section(cx: &mut App) -> Entity<ClockSection> {
    let service = Arc::new(ClockService::new(Arc::new(SystemClock)));
    let section = cx.new(|_| ClockSection::new(service.snapshot()));
    let weak_section = section.downgrade();
    let observer = cx.spawn(async move |cx| {
        loop {
            cx.background_executor()
                .spawn(async { thread::sleep(REFRESH_INTERVAL) })
                .await;
            let snapshot = service.snapshot();
            if weak_section
                .update(cx, |section, cx| section.apply(snapshot, cx))
                .is_err()
            {
                break;
            }
        }
    });
    section.update(cx, |section, _| section.own_observer(observer));
    section
}
