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
    pub known: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JoinDecision {
    Connect,
    RequirePassword,
}

pub fn decide_join(network: &WifiNetwork) -> JoinDecision {
    if network.secure && !network.known {
        JoinDecision::RequirePassword
    } else {
        JoinDecision::Connect
    }
}
