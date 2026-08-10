use gpui::Rgba;

use crate::core::workspace::HUE_SLOT_COUNT;

pub fn bar() -> Rgba {
    theme::core::palette::popup_background()
}

pub fn raise() -> Rgba {
    theme::core::palette::selection()
}

pub fn raise_faint() -> Rgba {
    theme::core::palette::raise_faint()
}

pub fn border() -> Rgba {
    theme::core::palette::border()
}

pub fn muted() -> Rgba {
    theme::core::palette::muted()
}

pub fn active_border() -> Rgba {
    Rgba {
        a: 0.28,
        ..theme::core::palette::subtle()
    }
}

pub fn text() -> Rgba {
    theme::core::palette::text()
}

pub fn text_bright() -> Rgba {
    theme::core::palette::text_bright()
}

pub fn ink() -> Rgba {
    theme::core::palette::ink()
}

pub fn accent() -> Rgba {
    theme::core::palette::accent()
}

pub fn transparent() -> Rgba {
    theme::core::palette::transparent()
}

pub fn hue(slot: usize) -> Rgba {
    const SLOTS: [usize; HUE_SLOT_COUNT] = [8, 9, 10, 11, 12, 13, 14, 15];
    theme::core::palette::slot(SLOTS[slot % SLOTS.len()])
}

pub fn hue_faint(slot: usize) -> Rgba {
    Rgba {
        a: 0.6,
        ..hue(slot)
    }
}
