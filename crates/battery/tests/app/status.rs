use std::sync::Arc;

use battery::app::status::BatteryService;
use battery::core::status::BatteryStatus;
use battery::ports::power::{BatteryReadError, BatterySource};

struct FixedBatterySource(Option<BatteryStatus>);

impl BatterySource for FixedBatterySource {
    fn read_status(&self) -> Result<Option<BatteryStatus>, BatteryReadError> {
        Ok(self.0)
    }
}

#[test]
fn load_returns_the_source_snapshot() {
    let expected = BatteryStatus::from_capacity(3, 4, true, true);
    let service = BatteryService::new(Arc::new(FixedBatterySource(expected)));

    assert_eq!(service.load().unwrap(), expected);
}
