use std::sync::Arc;

use crate::core::preferences::ThemePreferences;
use crate::ports::preferences::{PreferenceStore, PreferenceStoreError};

pub struct ThemeService {
    store: Arc<dyn PreferenceStore>,
}

impl ThemeService {
    pub fn new(store: Arc<dyn PreferenceStore>) -> Self {
        Self { store }
    }

    pub fn load(&self) -> Result<ThemePreferences, PreferenceStoreError> {
        self.store
            .load()
            .map(|preferences| preferences.unwrap_or_default().normalized())
    }

    pub fn save(&self, preferences: ThemePreferences) -> Result<(), PreferenceStoreError> {
        self.store.save(preferences.normalized())
    }
}
