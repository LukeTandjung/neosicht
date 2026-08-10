#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MusicApplication {
    Spotify,
    AppleMusic,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlaybackState {
    Playing,
    Paused,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NowPlaying {
    pub application: MusicApplication,
    pub state: PlaybackState,
    pub title: String,
    pub artist: String,
    pub artwork_url: Option<String>,
    pub embedded_artwork: Option<Artwork>,
    pub position_seconds: f64,
    pub duration_seconds: f64,
}

impl NowPlaying {
    pub fn progress_percentage(&self) -> f64 {
        if self.duration_seconds <= 0.0 {
            return 0.0;
        }
        (self.position_seconds / self.duration_seconds * 100.0).clamp(0.0, 100.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArtworkFormat {
    Jpeg,
    Png,
    Webp,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Artwork {
    pub format: ArtworkFormat,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportAction {
    Previous,
    TogglePlayback,
    Next,
}
