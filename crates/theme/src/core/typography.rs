//! The active font families, mirroring the design's `--ui`/`--mono` variables.
//! Hard-coded to the Plex set until the theme engine (phase 2) makes it
//! dynamic.

/// Monospace family for numerals, shortcuts, and small tracking labels.
pub fn mono() -> &'static str {
    "JetBrains Mono"
}
