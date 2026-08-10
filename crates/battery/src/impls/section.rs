use std::sync::Arc;
use std::thread;
use std::time::Duration;

use gpui::{App, AppContext, Entity};

use crate::adapters::iokit::IokitBatterySource;
use crate::app::status::BatteryService;
use crate::ui::section::BatterySection;

const REFRESH_INTERVAL: Duration = Duration::from_secs(5);

pub fn iokit_section(cx: &mut App) -> Entity<BatterySection> {
    let service = Arc::new(BatteryService::new(Arc::new(IokitBatterySource)));
    let section = cx.new(|_| BatterySection::new());
    let weak_section = section.downgrade();
    let observer = cx.spawn(async move |cx| {
        let mut initial = true;
        loop {
            let service = service.clone();
            let observed = cx
                .background_executor()
                .spawn(async move {
                    if !initial {
                        thread::sleep(REFRESH_INTERVAL);
                    }
                    service.load()
                })
                .await;
            initial = false;
            if weak_section
                .update(cx, |section, cx| section.apply(observed, cx))
                .is_err()
            {
                break;
            }
        }
    });
    section.update(cx, |section, _| section.own_observer(observer));
    section
}
