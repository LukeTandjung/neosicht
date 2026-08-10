use music::core::playback::{MusicApplication, NowPlaying, PlaybackState};

fn track(position_seconds: f64, duration_seconds: f64) -> NowPlaying {
    NowPlaying {
        application: MusicApplication::Spotify,
        state: PlaybackState::Playing,
        title: "Track".to_owned(),
        artist: "Artist".to_owned(),
        artwork_url: None,
        embedded_artwork: None,
        position_seconds,
        duration_seconds,
    }
}

#[test]
fn progress_is_normalized_and_clamped() {
    assert_eq!(track(30.0, 120.0).progress_percentage(), 25.0);
    assert_eq!(track(130.0, 120.0).progress_percentage(), 100.0);
    assert_eq!(track(10.0, 0.0).progress_percentage(), 0.0);
}
