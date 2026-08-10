/// Battery state independent of IOKit and presentation concerns.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BatteryStatus {
    percentage: u8,
    charging: bool,
    plugged_in: bool,
}

impl BatteryStatus {
    /// Converts source capacities into a normalized percentage. A source with
    /// no usable maximum capacity is treated as unavailable.
    pub fn from_capacity(
        current_capacity: u32,
        maximum_capacity: u32,
        charging: bool,
        plugged_in: bool,
    ) -> Option<Self> {
        if maximum_capacity == 0 {
            return None;
        }

        let percentage = current_capacity
            .saturating_mul(100)
            .checked_div(maximum_capacity)?
            .min(100) as u8;

        Some(Self {
            percentage,
            charging,
            plugged_in,
        })
    }

    pub fn percentage(self) -> u8 {
        self.percentage
    }

    pub fn is_charging(self) -> bool {
        self.charging
    }

    pub fn is_plugged_in(self) -> bool {
        self.plugged_in
    }
}
