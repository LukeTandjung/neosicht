use crate::core::catalog::{self, ACCENT_SLOTS};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemePreferences {
    pub light: bool,
    pub theme: usize,
    pub font: usize,
    pub accent: usize,
}

impl Default for ThemePreferences {
    fn default() -> Self {
        Self {
            light: false,
            theme: 0,
            font: 0,
            accent: 4,
        }
    }
}

impl ThemePreferences {
    pub fn normalized(self) -> Self {
        let theme = self.theme.min(catalog::catalog().len().saturating_sub(1));
        Self {
            light: self.light && catalog::catalog()[theme].light.is_some(),
            theme,
            font: self.font.min(catalog::font_sets().len().saturating_sub(1)),
            accent: self.accent.min(ACCENT_SLOTS.len().saturating_sub(1)),
        }
    }
}
