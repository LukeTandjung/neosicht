use crate::core::preferences::ThemePreferences;

#[derive(Debug)]
pub enum PreferenceStoreError {
    Unavailable(String),
    Malformed(String),
}

pub trait PreferenceStore: Send + Sync {
    fn load(&self) -> Result<Option<ThemePreferences>, PreferenceStoreError>;
    fn save(&self, preferences: ThemePreferences) -> Result<(), PreferenceStoreError>;
}
