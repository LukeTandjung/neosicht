use weather::core::forecast::{Coordinates, condition_name};

#[test]
fn coordinates_reject_values_outside_the_globe() {
    assert!(Coordinates::new(90.0, 180.0).is_some());
    assert!(Coordinates::new(90.1, 0.0).is_none());
    assert!(Coordinates::new(0.0, -180.1).is_none());
}

#[test]
fn wmo_codes_have_stable_display_names() {
    assert_eq!(condition_name(0), "Clear");
    assert_eq!(condition_name(63), "Rain");
    assert_eq!(condition_name(95), "Thunderstorm");
}
