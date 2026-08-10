use std::ffi::{CStr, c_char, c_double, c_int};

use serde::Deserialize;

use crate::core::event::CalendarEvent;
use crate::ports::calendar::{CalendarReadError, CalendarSource};

unsafe extern "C" {
    fn neosicht_calendar_events(
        starts_at: c_double,
        ends_before: c_double,
        error_code: *mut c_int,
    ) -> *mut c_char;
    fn neosicht_free_string(value: *mut c_char);
}

#[derive(Deserialize)]
struct EventDto {
    title: String,
    starts_at: f64,
    all_day: bool,
    calendar_name: String,
    #[serde(default)]
    calendar_color: Option<u32>,
}

/// EventKit-backed calendar reader. Objective-C and EventKit values never
/// cross its native scalar/UTF-8 boundary.
pub struct EventKitCalendarSource;

impl CalendarSource for EventKitCalendarSource {
    fn events_between(
        &self,
        starts_at: i64,
        ends_before: i64,
    ) -> Result<Vec<CalendarEvent>, CalendarReadError> {
        let mut error_code = 0;
        // SAFETY: error_code is writable and the returned pointer is either
        // null or an owned NUL-terminated allocation from the paired shim.
        let json = unsafe {
            neosicht_calendar_events(starts_at as f64, ends_before as f64, &mut error_code)
        };
        if json.is_null() {
            return Err(match error_code {
                1 => CalendarReadError::PermissionDenied,
                2 => CalendarReadError::Unavailable,
                _ => CalendarReadError::MalformedEvent,
            });
        }

        // SAFETY: the native contract guarantees valid NUL-terminated UTF-8.
        let bytes = unsafe { CStr::from_ptr(json) }.to_bytes();
        let decoded = serde_json::from_slice::<Vec<EventDto>>(bytes)
            .map_err(|_| CalendarReadError::MalformedEvent);
        // SAFETY: json came from neosicht_calendar_events and is released once.
        unsafe { neosicht_free_string(json) };

        decoded.map(|events| {
            events
                .into_iter()
                .map(|event| CalendarEvent {
                    title: event.title,
                    starts_at: event.starts_at.round() as i64,
                    all_day: event.all_day,
                    calendar_name: event.calendar_name,
                    calendar_color: event.calendar_color,
                })
                .collect()
        })
    }
}
