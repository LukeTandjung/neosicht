//! The active palette every bar section draws from. Hard-coded to Tokyo Night
//! dark until the base16 theme engine (phase 2) makes it dynamic; the accessor
//! names mirror the design's CSS variables so sections read like the design.

use gpui::{Rgba, rgb, rgba};

/// Desktop background / darkest surface (`--bg`).
pub fn bg() -> Rgba {
    rgb(0x1a1b26)
}

/// Bar and popover surface (`--bar`).
pub fn bar() -> Rgba {
    rgb(0x24283b)
}

/// Raised surface for hovers and selected rows (`--raise`).
pub fn raise() -> Rgba {
    rgb(0x2f3549)
}

/// The resting (inactive) fill of chips and pills: `raise` at 45% alpha.
pub fn raise_faint() -> Rgba {
    rgba(0x2f354973)
}

/// Hairline borders (`--bd`).
pub fn border() -> Rgba {
    rgb(0x2f3549)
}

/// Disabled / decorative ink (`--mut`).
pub fn muted() -> Rgba {
    rgb(0x444b6a)
}

/// Secondary text (`--sub`).
pub fn subtle() -> Rgba {
    rgb(0x787c99)
}

/// Body text (`--fg`).
pub fn text() -> Rgba {
    rgb(0xa9b1d6)
}

/// Emphasized text (`--fgb`).
pub fn text_bright() -> Rgba {
    rgb(0xc0caf5)
}

/// Dark ink for text drawn on accent-colored surfaces.
pub fn ink() -> Rgba {
    rgb(0x1a1b26)
}

/// The user-selected accent slot (`--acc`), currently blue.
pub fn accent() -> Rgba {
    rgb(0x7aa2f7)
}

pub fn red() -> Rgba {
    rgb(0xf7768e)
}

pub fn orange() -> Rgba {
    rgb(0xff9e64)
}

pub fn yellow() -> Rgba {
    rgb(0xe0af68)
}

pub fn green() -> Rgba {
    rgb(0x9ece6a)
}

pub fn cyan() -> Rgba {
    rgb(0x2ac3de)
}

pub fn blue() -> Rgba {
    rgb(0x7aa2f7)
}

pub fn magenta() -> Rgba {
    rgb(0xbb9af7)
}

pub fn transparent() -> Rgba {
    rgba(0x00000000)
}

/// Full-window dim behind modal dialogs.
pub fn scrim() -> Rgba {
    rgba(0x08080c80)
}

/// One of the sixteen base16 slots of the active palette.
pub fn slot(index: usize) -> Rgba {
    rgb(crate::core::catalog::catalog()[0].dark[index % 16])
}
