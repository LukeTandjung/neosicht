use std::sync::{Arc, Mutex};

use music::app::player::PlayerService;
use music::core::playback::{Artwork, MusicApplication, NowPlaying, TransportAction};
use music::ports::player::{ArtworkSource, PlayerError, PlayerSource};

struct EmptyArtwork;

impl ArtworkSource for EmptyArtwork {
    fn load(&self, _url: &str) -> Result<Artwork, PlayerError> {
        Err(PlayerError::Unavailable)
    }
}

struct RecordingPlayer {
    actions: Arc<Mutex<Vec<(MusicApplication, TransportAction)>>>,
}

impl PlayerSource for RecordingPlayer {
    fn now_playing(&self) -> Result<Option<NowPlaying>, PlayerError> {
        Ok(None)
    }

    fn perform(
        &self,
        application: MusicApplication,
        action: TransportAction,
    ) -> Result<(), PlayerError> {
        self.actions.lock().unwrap().push((application, action));
        Ok(())
    }
}

#[test]
fn transport_actions_are_routed_to_the_selected_application() {
    let actions = Arc::new(Mutex::new(Vec::new()));
    let service = PlayerService::new(
        Arc::new(RecordingPlayer {
            actions: actions.clone(),
        }),
        Arc::new(EmptyArtwork),
    );

    service
        .perform(MusicApplication::AppleMusic, TransportAction::Next)
        .unwrap();

    assert_eq!(
        *actions.lock().unwrap(),
        [(MusicApplication::AppleMusic, TransportAction::Next)]
    );
}
