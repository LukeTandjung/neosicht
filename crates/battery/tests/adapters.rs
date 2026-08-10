#[cfg(target_os = "macos")]
#[test]
fn iokit_power_source_can_be_queried() {
    use battery::adapters::iokit::IokitBatterySource;
    use battery::ports::power::BatterySource;

    let status = IokitBatterySource
        .read_status()
        .expect("IOKit query should succeed");
    assert!(status.is_some(), "a Mac battery should be discovered");
}
