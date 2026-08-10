use std::sync::Arc;

use chrono::NaiveDate;

use crate::ports::clock::Clock;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClockSnapshot {
    pub display: String,
    pub local_date: NaiveDate,
}

/// Hides wall-clock access and presentation formatting behind one snapshot operation.
pub struct ClockService {
    clock: Arc<dyn Clock>,
}

impl ClockService {
    pub fn new(clock: Arc<dyn Clock>) -> Self {
        Self { clock }
    }

    pub fn snapshot(&self) -> ClockSnapshot {
        let now = self.clock.now();
        ClockSnapshot {
            display: now.format("%a %b %-d %H:%M").to_string(),
            local_date: now.date_naive(),
        }
    }
}
