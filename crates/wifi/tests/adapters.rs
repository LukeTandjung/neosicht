use wifi::adapters::core_wlan::CoreWlanProvider;
use wifi::ports::wifi::WifiProvider;

#[test]
fn core_wlan_snapshot_can_be_queried() {
    let snapshot = CoreWlanProvider.observe().expect("CoreWLAN snapshot");
    assert!(
        snapshot
            .networks
            .iter()
            .all(|network| !network.ssid.is_empty())
    );
}
