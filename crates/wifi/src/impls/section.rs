use std::sync::Arc;
use std::thread;
use std::time::Duration;

use gpui::{App, AppContext, Entity};

use crate::adapters::core_wlan::CoreWlanProvider;
use crate::app::wifi::WifiService;
use crate::ui::section::WifiSection;

pub fn core_wlan_section(cx: &mut App) -> Entity<WifiSection> {
    let service = Arc::new(WifiService::new(Arc::new(CoreWlanProvider)));
    let section = cx.new(|cx| WifiSection::new(service.clone(), cx));
    let weak = section.downgrade();
    cx.spawn(async move |cx| {
        let mut initial = true;
        loop {
            let service = service.clone();
            let observed = cx
                .background_executor()
                .spawn(async move {
                    if !initial {
                        thread::sleep(Duration::from_secs(5));
                    }
                    service.observe()
                })
                .await;
            initial = false;
            if weak
                .update(cx, |section, cx| section.apply(observed, cx))
                .is_err()
            {
                break;
            }
        }
    })
    .detach();
    section
}
