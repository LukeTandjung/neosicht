use battery::core::status::BatteryStatus;

#[test]
fn capacity_is_normalized_to_a_percentage() {
    let status = BatteryStatus::from_capacity(36, 50, false, true).unwrap();

    assert_eq!(status.percentage(), 72);
    assert!(!status.is_charging());
    assert!(status.is_plugged_in());
}

#[test]
fn overreported_capacity_is_clamped() {
    let status = BatteryStatus::from_capacity(60, 50, true, true).unwrap();

    assert_eq!(status.percentage(), 100);
}

#[test]
fn zero_maximum_capacity_is_unavailable() {
    assert_eq!(BatteryStatus::from_capacity(10, 0, false, false), None);
}
