use std::sync::atomic::{AtomicUsize, Ordering};

use crate::core::catalog;

static FONT: AtomicUsize = AtomicUsize::new(0);

pub fn activate(font: usize) {
    FONT.store(
        font.min(catalog::font_sets().len().saturating_sub(1)),
        Ordering::Relaxed,
    );
}

pub fn ui() -> &'static str {
    catalog::font_sets()[FONT.load(Ordering::Relaxed)].ui_family
}

pub fn mono() -> &'static str {
    catalog::font_sets()[FONT.load(Ordering::Relaxed)].mono_family
}
