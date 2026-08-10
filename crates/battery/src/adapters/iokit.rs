use std::ffi::c_int;

use crate::core::status::BatteryStatus;
use crate::ports::power::{BatteryReadError, BatterySource};

unsafe extern "C" {
    fn neosicht_read_battery(
        current_capacity: *mut c_int,
        maximum_capacity: *mut c_int,
        charging: *mut bool,
        plugged_in: *mut bool,
    ) -> c_int;
}

/// Reads the primary macOS power source while keeping Core Foundation and
/// Objective-C values inside the native adapter.
pub struct IokitBatterySource;

impl BatterySource for IokitBatterySource {
    fn read_status(&self) -> Result<Option<BatteryStatus>, BatteryReadError> {
        let mut current_capacity = 0;
        let mut maximum_capacity = 0;
        let mut charging = false;
        let mut plugged_in = false;

        // SAFETY: every pointer refers to writable storage for the duration of
        // the call; the native shim writes only the declared scalar values.
        let result = unsafe {
            neosicht_read_battery(
                &mut current_capacity,
                &mut maximum_capacity,
                &mut charging,
                &mut plugged_in,
            )
        };

        match result {
            0 => Ok(None),
            1 => {
                let current_capacity = u32::try_from(current_capacity)
                    .map_err(|_| BatteryReadError::MalformedSource)?;
                let maximum_capacity = u32::try_from(maximum_capacity)
                    .map_err(|_| BatteryReadError::MalformedSource)?;
                Ok(BatteryStatus::from_capacity(
                    current_capacity,
                    maximum_capacity,
                    charging,
                    plugged_in,
                ))
            }
            _ => Err(BatteryReadError::SourceUnavailable),
        }
    }
}
