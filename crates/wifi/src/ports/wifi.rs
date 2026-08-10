use crate::core::network::WifiSnapshot;

#[derive(Debug)]
pub enum WifiError {
    Unavailable(String),
    Failed(String),
    Malformed(String),
}

pub trait WifiProvider: Send + Sync {
    fn observe(&self) -> Result<WifiSnapshot, WifiError>;
    fn set_enabled(&self, enabled: bool) -> Result<(), WifiError>;
    fn join(&self, ssid: &str, password: Option<&str>) -> Result<(), WifiError>;
}
