use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use gpui::{Rgba, rgb, rgba};

use crate::core::catalog::{self, ACCENT_SLOTS};
use crate::core::preferences::ThemePreferences;

static LIGHT: AtomicBool = AtomicBool::new(false);
static THEME: AtomicUsize = AtomicUsize::new(0);
static ACCENT: AtomicUsize = AtomicUsize::new(4);

pub fn activate(preferences: ThemePreferences) {
    let preferences = preferences.normalized();
    LIGHT.store(preferences.light, Ordering::Relaxed);
    THEME.store(preferences.theme, Ordering::Relaxed);
    ACCENT.store(preferences.accent, Ordering::Relaxed);
}

fn active_slots() -> &'static [u32; 16] {
    catalog::catalog()[THEME.load(Ordering::Relaxed)].swatches(LIGHT.load(Ordering::Relaxed))
}

pub fn bg() -> Rgba {
    rgb(active_slots()[0])
}

pub fn bar() -> Rgba {
    rgb(active_slots()[1])
}

pub fn raise() -> Rgba {
    rgb(active_slots()[2])
}

pub fn raise_faint() -> Rgba {
    let color = active_slots()[2];
    rgba((color << 8) | 0x73)
}

pub fn border() -> Rgba {
    rgb(active_slots()[2])
}

/// Base16 base00: the Stylix default surface for overlays.
pub fn popup_background() -> Rgba {
    rgb(active_slots()[0])
}

/// Base16 base02: the Stylix selection background.
pub fn selection() -> Rgba {
    rgb(active_slots()[2])
}

pub fn muted() -> Rgba {
    rgb(active_slots()[3])
}

pub fn subtle() -> Rgba {
    rgb(active_slots()[4])
}

pub fn text() -> Rgba {
    rgb(active_slots()[5])
}

pub fn text_bright() -> Rgba {
    rgb(active_slots()[6])
}

pub fn ink() -> Rgba {
    rgb(active_slots()[0])
}

pub fn accent() -> Rgba {
    rgb(active_slots()[ACCENT_SLOTS[ACCENT.load(Ordering::Relaxed)]])
}

pub fn red() -> Rgba {
    rgb(active_slots()[8])
}

pub fn orange() -> Rgba {
    rgb(active_slots()[9])
}

pub fn yellow() -> Rgba {
    rgb(active_slots()[10])
}

pub fn green() -> Rgba {
    rgb(active_slots()[11])
}

pub fn cyan() -> Rgba {
    rgb(active_slots()[12])
}

pub fn blue() -> Rgba {
    rgb(active_slots()[13])
}

pub fn magenta() -> Rgba {
    rgb(active_slots()[14])
}

pub fn transparent() -> Rgba {
    rgba(0x00000000)
}

pub fn scrim() -> Rgba {
    rgba(0x08080c80)
}

pub fn slot(index: usize) -> Rgba {
    rgb(active_slots()[index % 16])
}
