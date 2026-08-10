use crate::core::status::BatteryStatus;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BatteryReadError {
    SourceUnavailable,
    MalformedSource,
}

/// Supplies the computer's current battery state. Platform handles and source
/// dictionaries remain private to the adapter implementing this contract.
pub trait BatterySource: Send + Sync {
    fn read_status(&self) -> Result<Option<BatteryStatus>, BatteryReadError>;
}
