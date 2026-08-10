use std::sync::Arc;
use std::thread;
use std::time::Duration;

use gpui::{App, AppContext, Entity};

use crate::adapters::core_location::CoreLocationSource;
use crate::adapters::open_meteo::OpenMeteoForecastSource;
use crate::app::weather::WeatherService;
use crate::ui::section::WeatherSection;

const REFRESH_INTERVAL: Duration = Duration::from_secs(15 * 60);

pub fn macos_section(cx: &mut App) -> Entity<WeatherSection> {
    let service = Arc::new(WeatherService::new(
        Arc::new(CoreLocationSource),
        Arc::new(OpenMeteoForecastSource::new()),
    ));
    let section = cx.new(|_| WeatherSection::new());
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
                    service.load()
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
