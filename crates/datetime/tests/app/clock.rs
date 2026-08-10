use std::sync::Arc;

use chrono::{DateTime, Local, TimeZone as _};
use datetime::app::clock::ClockService;
use datetime::ports::clock::Clock;

struct FixedClock(DateTime<Local>);

impl Clock for FixedClock {
    fn now(&self) -> DateTime<Local> {
        self.0
    }
}

#[test]
fn snapshot_formats_an_injected_local_time() {
    let instant = Local.with_ymd_and_hms(2026, 1, 15, 9, 7, 0).unwrap();
    let service = ClockService::new(Arc::new(FixedClock(instant)));

    let snapshot = service.snapshot();

    assert_eq!(snapshot.display, "Thu Jan 15 09:07");
    assert_eq!(snapshot.local_date.to_string(), "2026-01-15");
}
