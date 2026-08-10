use theme::core::catalog::{self, ACCENT_SLOTS};
use theme::core::preferences::ThemePreferences;

#[test]
fn preferences_are_normalized_to_the_catalog() {
    let preferences = ThemePreferences {
        light: true,
        theme: usize::MAX,
        font: usize::MAX,
        accent: usize::MAX,
    }
    .normalized();

    assert_eq!(preferences.theme, catalog::catalog().len() - 1);
    assert_eq!(preferences.font, catalog::font_sets().len() - 1);
    assert_eq!(preferences.accent, ACCENT_SLOTS.len() - 1);
    assert!(preferences.light);
}

#[test]
fn dark_only_themes_disable_light_appearance() {
    let nord = catalog::catalog()
        .iter()
        .position(|theme| theme.name == "Nord")
        .expect("Nord is in the catalog");

    let preferences = ThemePreferences {
        light: true,
        theme: nord,
        ..ThemePreferences::default()
    }
    .normalized();

    assert!(!preferences.light);
}
