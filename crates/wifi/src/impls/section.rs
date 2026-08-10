use std::sync::Arc;
use std::thread;
use std::time::Duration;

use gpui::{App, AppContext, Entity};

use crate::adapters::core_wlan::CoreWlanProvider;
use crate::ports::wifi::WifiProvider;
use crate::ui::section::{SectionEvent, WifiSection};

pub fn core_wlan_section(cx: &mut App) -> Entity<WifiSection> {
    let service = Arc::new(CoreWlanProvider);
    let section = cx.new(WifiSection::new);
    let weak = section.downgrade();
    let action_service = service.clone();
    let action_section = section.clone();
    cx.subscribe(&section, move |_section, event, cx| {
        let (ssid, password, remember) = match event {
            SectionEvent::Join {
                ssid,
                password,
                remember,
            } => (ssid.clone(), password.clone(), *remember),
            SectionEvent::SetEnabled { enabled } => {
                let service = action_service.clone();
                let weak = action_section.downgrade();
                let enabled = *enabled;
                let operation = cx.spawn(async move |cx| {
                    let result = cx
                        .background_executor()
                        .spawn(async move { service.set_enabled(enabled) })
                        .await;
                    weak.update(cx, |section, cx| section.apply_enabled(enabled, result, cx))
                        .ok();
                });
                action_section.update(cx, |section, _| section.own_operation(operation));
                return;
            }
            SectionEvent::PopupExtentChanged { .. } => return,
        };
        let service = action_service.clone();
        let weak = action_section.downgrade();
        let requested_ssid = ssid.clone();
        let operation = cx.spawn(async move |cx| {
            let result = cx
                .background_executor()
                .spawn(async move { service.join(&requested_ssid, password.as_deref(), remember) })
                .await;
            weak.update(cx, |section, cx| section.apply_connected(ssid, result, cx))
                .ok();
        });
        action_section.update(cx, |section, _| section.own_operation(operation));
    })
    .detach();
    let observer = cx.spawn(async move |cx| {
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
    });
    section.update(cx, |section, _| section.own_observer(observer));
    section
}
