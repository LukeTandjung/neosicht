#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WifiSnapshot {
    pub enabled: bool,
    pub connected_ssid: Option<String>,
    pub networks: Vec<WifiNetwork>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal: i32,
    pub secure: bool,
}
