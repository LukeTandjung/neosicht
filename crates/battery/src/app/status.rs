use crate::core::status::BatteryStatus;
use crate::ports::power::{BatteryReadError, BatterySource};
use std::sync::Arc;

/// Reads a player-neutral battery snapshot while hiding the power source.
pub struct BatteryService {
    source: Arc<dyn BatterySource>,
}

impl BatteryService {
    pub fn new(source: Arc<dyn BatterySource>) -> Self {
        Self { source }
    }

    pub fn load(&self) -> Result<Option<BatteryStatus>, BatteryReadError> {
        self.source.read_status()
    }
}
