use wifi::core::network::{JoinDecision, WifiNetwork, decide_join};

#[test]
fn unknown_secured_network_requires_password() {
    let network = WifiNetwork {
        ssid: "secured".to_owned(),
        signal: -50,
        secure: true,
        known: false,
    };
    assert_eq!(decide_join(&network), JoinDecision::RequirePassword);
}

#[test]
fn open_and_known_networks_connect_directly() {
    for network in [
        WifiNetwork {
            ssid: "open".to_owned(),
            signal: -50,
            secure: false,
            known: false,
        },
        WifiNetwork {
            ssid: "known".to_owned(),
            signal: -50,
            secure: true,
            known: true,
        },
    ] {
        assert_eq!(decide_join(&network), JoinDecision::Connect);
    }
}
