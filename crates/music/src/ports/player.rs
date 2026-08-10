use crate::core::playback::{Artwork, MusicApplication, NowPlaying, TransportAction};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlayerError {
    AutomationDenied,
    Unavailable,
    MalformedResponse,
}

/// Observes and controls supported players without exposing AppleScript or
/// application-specific command syntax.
pub trait ArtworkSource: Send + Sync {
    fn load(&self, url: &str) -> Result<Artwork, PlayerError>;
}

pub trait PlayerSource: Send + Sync {
    fn now_playing(&self) -> Result<Option<NowPlaying>, PlayerError>;

    fn perform(
        &self,
        application: MusicApplication,
        action: TransportAction,
    ) -> Result<(), PlayerError>;
}
