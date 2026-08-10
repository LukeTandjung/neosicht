use std::sync::Arc;

use crate::core::network::WifiSnapshot;
use crate::ports::wifi::{WifiError, WifiProvider};

pub struct WifiService {
    provider: Arc<dyn WifiProvider>,
}

impl WifiService {
    pub fn new(provider: Arc<dyn WifiProvider>) -> Self {
        Self { provider }
    }

    pub fn observe(&self) -> Result<WifiSnapshot, WifiError> {
        self.provider.observe()
    }

    pub fn set_enabled(&self, enabled: bool) -> Result<(), WifiError> {
        self.provider.set_enabled(enabled)
    }

    pub fn join(&self, ssid: &str, password: Option<&str>) -> Result<(), WifiError> {
        self.provider.join(ssid, password)
    }
}
