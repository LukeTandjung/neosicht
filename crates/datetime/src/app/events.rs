use crate::core::event::CalendarEvent;
use crate::ports::calendar::{CalendarReadError, CalendarSource};
use std::sync::Arc;

const EVENT_WINDOW_SECONDS: i64 = 7 * 24 * 60 * 60;
const EVENT_LIMIT: usize = 3;
/// Loads the compact upcoming-event view while hiding EventKit query and
/// result-shaping details from the UI.
pub struct UpcomingEventsService {
    source: Arc<dyn CalendarSource>,
}

impl UpcomingEventsService {
    pub fn new(source: Arc<dyn CalendarSource>) -> Self {
        Self { source }
    }

    pub fn load(&self, now: i64) -> Result<Vec<CalendarEvent>, CalendarReadError> {
        let mut events = self
            .source
            .events_between(now, now.saturating_add(EVENT_WINDOW_SECONDS))?;
        events.sort_by_key(|event| event.starts_at);
        events.truncate(EVENT_LIMIT);
        Ok(events)
    }
}
