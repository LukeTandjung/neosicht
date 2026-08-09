//! Hard-coded Tokyo Night values matching the shell bar until the base16
//! theme engine (phase 2) replaces them.

use gpui::{Rgba, rgb, rgba};

use crate::core::workspace::HUE_SLOT_COUNT;

pub fn bar() -> Rgba {
    rgb(0x24283b)
}

pub fn raise() -> Rgba {
    rgb(0x2f3549)
}

/// The resting (inactive) fill of chips and pills: `raise` at 45% alpha.
pub fn raise_faint() -> Rgba {
    rgba(0x2f354973)
}

pub fn border() -> Rgba {
    rgb(0x2f3549)
}

pub fn muted() -> Rgba {
    rgb(0x444b6a)
}

/// Border of the active pill: subtle text color at 28% alpha.
pub fn active_border() -> Rgba {
    rgba(0x787c9948)
}

pub fn text() -> Rgba {
    rgb(0xa9b1d6)
}

pub fn text_bright() -> Rgba {
    rgb(0xc0caf5)
}

/// Dark ink for text drawn on accent-colored surfaces.
pub fn ink() -> Rgba {
    rgb(0x1a1b26)
}

pub fn accent() -> Rgba {
    rgb(0x7aa2f7)
}

pub fn transparent() -> Rgba {
    rgba(0x00000000)
}

/// The eight accent hues application tiles and workspace numbers draw from.
pub fn hue(slot: usize) -> Rgba {
    const HUES: [u32; HUE_SLOT_COUNT] = [
        0xf7768e, // red
        0xff9e64, // orange
        0xe0af68, // yellow
        0x9ece6a, // green
        0x73daca, // teal
        0x2ac3de, // cyan
        0x7aa2f7, // blue
        0xbb9af7, // magenta
    ];
    rgb(HUES[slot % HUES.len()])
}

pub fn hue_faint(slot: usize) -> Rgba {
    Rgba {
        a: 0.6,
        ..hue(slot)
    }
}
