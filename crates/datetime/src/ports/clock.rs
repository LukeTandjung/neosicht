use chrono::{DateTime, Local};

/// Supplies local wall-clock time without coupling application code to the system clock.
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Local>;
}
