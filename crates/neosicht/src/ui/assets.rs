//! The shell's asset source: aggregates the icon SVGs each widget crate
//! embeds so gpui's `svg()` elements can resolve them by path.

use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};

pub struct ShellAssets;

fn sources() -> impl Iterator<Item = (&'static str, &'static [u8])> {
    notifications::ui::section::ASSETS
        .into_iter()
        .chain(volume::ui::section::ASSETS)
        .chain(bluetooth::ui::section::ASSETS)
        .chain(wifi::ui::section::ASSETS)
        .chain(wallpaper::ui::section::ASSETS)
}

impl AssetSource for ShellAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(sources()
            .find(|(source_path, _)| *source_path == path)
            .map(|(_, bytes)| Cow::Borrowed(bytes)))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(sources()
            .filter(|(source_path, _)| source_path.starts_with(path))
            .map(|(source_path, _)| SharedString::from(source_path))
            .collect())
    }
}
