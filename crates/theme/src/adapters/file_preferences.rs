use std::fs;
use std::path::PathBuf;

use crate::core::preferences::ThemePreferences;
use crate::ports::preferences::{PreferenceStore, PreferenceStoreError};

pub struct FilePreferenceStore {
    path: PathBuf,
}

impl FilePreferenceStore {
    pub fn standard() -> Self {
        let root = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .unwrap_or_else(|| PathBuf::from("."));
        Self {
            path: root.join("neosicht/theme"),
        }
    }

    fn parse(contents: &str) -> Result<ThemePreferences, PreferenceStoreError> {
        let mut preferences = ThemePreferences::default();
        for line in contents.lines() {
            let Some((key, value)) = line.split_once('=') else {
                return Err(PreferenceStoreError::Malformed(line.to_owned()));
            };
            match key {
                "light" => preferences.light = value == "true",
                "theme" => preferences.theme = parse_index(key, value)?,
                "font" => preferences.font = parse_index(key, value)?,
                "accent" => preferences.accent = parse_index(key, value)?,
                _ => return Err(PreferenceStoreError::Malformed(key.to_owned())),
            }
        }
        Ok(preferences.normalized())
    }
}

fn parse_index(key: &str, value: &str) -> Result<usize, PreferenceStoreError> {
    value
        .parse()
        .map_err(|_| PreferenceStoreError::Malformed(format!("{key}={value}")))
}

impl PreferenceStore for FilePreferenceStore {
    fn load(&self) -> Result<Option<ThemePreferences>, PreferenceStoreError> {
        match fs::read_to_string(&self.path) {
            Ok(contents) => Self::parse(&contents).map(Some),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(PreferenceStoreError::Unavailable(error.to_string())),
        }
    }

    fn save(&self, preferences: ThemePreferences) -> Result<(), PreferenceStoreError> {
        let parent = self.path.parent().ok_or_else(|| {
            PreferenceStoreError::Unavailable("theme path has no parent".to_owned())
        })?;
        fs::create_dir_all(parent)
            .map_err(|error| PreferenceStoreError::Unavailable(error.to_string()))?;
        let temporary = self.path.with_extension("tmp");
        let contents = format!(
            "light={}\ntheme={}\nfont={}\naccent={}\n",
            preferences.light, preferences.theme, preferences.font, preferences.accent
        );
        fs::write(&temporary, contents)
            .and_then(|_| fs::rename(&temporary, &self.path))
            .map_err(|error| PreferenceStoreError::Unavailable(error.to_string()))
    }
}
