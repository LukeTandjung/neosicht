use std::sync::Arc;
use std::thread;
use std::time::Duration;

use gpui::{App, AppContext, Entity};

use crate::adapters::applescript::AppleScriptPlayerSource;
use crate::adapters::artwork::HttpArtworkSource;
use crate::app::player::PlayerService;
use crate::ui::section::MusicSection;

const REFRESH_INTERVAL: Duration = Duration::from_millis(500);

pub fn applescript_section(cx: &mut App) -> Entity<MusicSection> {
    let service = Arc::new(PlayerService::new(
        Arc::new(AppleScriptPlayerSource::new()),
        Arc::new(HttpArtworkSource::new()),
    ));
    let section = cx.new(|_| MusicSection::new(service.clone()));
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
