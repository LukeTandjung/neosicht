use crate::core::event::CalendarEvent;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CalendarReadError {
    PermissionDenied,
    Unavailable,
    MalformedEvent,
}

/// Reads events in a half-open Unix timestamp range. EventKit objects and
/// authorization mechanics remain private to the adapter.
pub trait CalendarSource: Send + Sync {
    fn events_between(
        &self,
        starts_at: i64,
        ends_before: i64,
    ) -> Result<Vec<CalendarEvent>, CalendarReadError>;
}
