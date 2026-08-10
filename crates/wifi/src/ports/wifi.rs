use crate::core::network::WifiSnapshot;

#[derive(Debug, PartialEq, Eq)]
pub enum WifiError {
    Unavailable(String),
    Failed(String),
    Malformed(String),
}

impl std::fmt::Display for WifiError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Unavailable(message) | Self::Failed(message) | Self::Malformed(message) => {
                message
            }
        };
        formatter.write_str(message)
    }
}

pub trait WifiProvider: Send + Sync {
    fn observe(&self) -> Result<WifiSnapshot, WifiError>;
    fn set_enabled(&self, enabled: bool) -> Result<(), WifiError>;
    fn join(&self, ssid: &str, password: Option<&str>, remember: bool) -> Result<(), WifiError>;
}
