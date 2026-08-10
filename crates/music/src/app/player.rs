use crate::core::playback::{Artwork, MusicApplication, NowPlaying, TransportAction};
use crate::ports::player::{ArtworkSource, PlayerError, PlayerSource};
use std::sync::Arc;

/// Owns player observation, refresh policy, and transport routing. The UI only
/// deals in player-neutral snapshots and actions.
pub struct PlayerService {
    source: Arc<dyn PlayerSource>,
    artwork: Arc<dyn ArtworkSource>,
}

impl PlayerService {
    pub fn new(source: Arc<dyn PlayerSource>, artwork: Arc<dyn ArtworkSource>) -> Self {
        Self { source, artwork }
    }

    pub fn load(&self) -> Result<Option<NowPlaying>, PlayerError> {
        self.source.now_playing()
    }

    pub fn load_artwork(&self, url: &str) -> Result<Artwork, PlayerError> {
        self.artwork.load(url)
    }

    pub fn perform(
        &self,
        application: MusicApplication,
        action: TransportAction,
    ) -> Result<(), PlayerError> {
        self.source.perform(application, action)
    }
}
