/// Calendar event data needed by the status island, independent of EventKit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalendarEvent {
    pub title: String,
    /// Seconds since the Unix epoch.
    pub starts_at: i64,
    pub all_day: bool,
    pub calendar_name: String,
    /// The calendar's user-assigned color as sRGB `0xRRGGBB`, when known.
    pub calendar_color: Option<u32>,
}
