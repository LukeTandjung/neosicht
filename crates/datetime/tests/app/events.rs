use std::sync::Arc;

use datetime::app::events::UpcomingEventsService;
use datetime::core::event::CalendarEvent;
use datetime::ports::calendar::{CalendarReadError, CalendarSource};

struct FixedCalendar(Vec<CalendarEvent>);

impl CalendarSource for FixedCalendar {
    fn events_between(
        &self,
        _starts_at: i64,
        _ends_before: i64,
    ) -> Result<Vec<CalendarEvent>, CalendarReadError> {
        Ok(self.0.clone())
    }
}

fn event(title: &str, starts_at: i64) -> CalendarEvent {
    CalendarEvent {
        title: title.to_owned(),
        starts_at,
        all_day: false,
        calendar_name: "Personal".to_owned(),
        calendar_color: None,
    }
}

#[test]
fn upcoming_events_are_ordered_and_limited_to_three() {
    let source = FixedCalendar(vec![
        event("Fourth", 40),
        event("Second", 20),
        event("First", 10),
        event("Third", 30),
    ]);
    let service = UpcomingEventsService::new(Arc::new(source));

    let titles: Vec<String> = service
        .load(0)
        .unwrap()
        .into_iter()
        .map(|event| event.title)
        .collect();

    assert_eq!(titles, ["First", "Second", "Third"]);
}
