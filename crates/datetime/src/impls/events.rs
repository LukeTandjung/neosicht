use std::sync::Arc;
use std::thread;
use std::time::Duration;

use chrono::Local;
use gpui::{App, AppContext, Entity};

use crate::adapters::eventkit::EventKitCalendarSource;
use crate::app::events::UpcomingEventsService;
use crate::ui::events::UpcomingEventsSection;

const REFRESH_INTERVAL: Duration = Duration::from_secs(60);

pub fn eventkit_section(cx: &mut App) -> Entity<UpcomingEventsSection> {
    let service = Arc::new(UpcomingEventsService::new(Arc::new(EventKitCalendarSource)));
    let section = cx.new(|_| UpcomingEventsSection::new());
    let weak_section = section.downgrade();
    let observer = cx.spawn(async move |cx| {
        let mut initial = true;
        loop {
            let service = service.clone();
            let loaded = cx
                .background_executor()
                .spawn(async move {
                    if !initial {
                        thread::sleep(REFRESH_INTERVAL);
                    }
                    service.load(Local::now().timestamp())
                })
                .await;
            initial = false;
            if weak_section
                .update(cx, |section, cx| section.apply(loaded, cx))
                .is_err()
            {
                break;
            }
        }
    });
    section.update(cx, |section, _| section.own_observer(observer));
    section
}
