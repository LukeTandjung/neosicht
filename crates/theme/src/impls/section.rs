use std::sync::Arc;

use gpui::{App, AppContext, Entity};

use crate::adapters::file_preferences::FilePreferenceStore;
use crate::app::preferences::ThemeService;
use crate::ui::section::ThemeSection;

pub fn theme_section(cx: &mut App) -> Entity<ThemeSection> {
    let service = Arc::new(ThemeService::new(Arc::new(FilePreferenceStore::standard())));
    cx.new(|cx| ThemeSection::new(service, cx))
}
