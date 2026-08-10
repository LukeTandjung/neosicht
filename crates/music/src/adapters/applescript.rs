use std::ffi::{CStr, c_char, c_int};
use std::sync::Mutex;

use base64::Engine as _;
use serde::Deserialize;

use crate::core::playback::{
    Artwork, ArtworkFormat, MusicApplication, NowPlaying, PlaybackState, TransportAction,
};
use crate::ports::player::{PlayerError, PlayerSource};

unsafe extern "C" {
    fn neosicht_now_playing(include_artwork: c_int, error_code: *mut c_int) -> *mut c_char;
    fn neosicht_music_transport(application: c_int, action: c_int) -> c_int;
    fn neosicht_music_free_string(value: *mut c_char);
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum ApplicationDto {
    Spotify,
    AppleMusic,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum StateDto {
    Playing,
    Paused,
}

#[derive(Deserialize)]
struct NowPlayingDto {
    application: ApplicationDto,
    state: StateDto,
    title: String,
    artist: String,
    artwork_url: String,
    artwork_data: String,
    position_seconds: f64,
    duration_seconds: f64,
}

/// AppleScript-backed Spotify and Apple Music integration. Script syntax,
/// descriptors, and Automation errors remain private to this adapter.
pub struct AppleScriptPlayerSource {
    artwork_track: Mutex<Option<String>>,
}

impl AppleScriptPlayerSource {
    pub fn new() -> Self {
        Self {
            artwork_track: Mutex::new(None),
        }
    }

    fn read(&self, include_artwork: bool) -> Result<Option<NowPlaying>, PlayerError> {
        let mut error_code = 0;
        // SAFETY: error_code is writable and a non-null result follows the
        // paired owned-string contract.
        let json = unsafe { neosicht_now_playing(i32::from(include_artwork), &mut error_code) };
        if json.is_null() {
            return match error_code {
                0 => Ok(None),
                1 => Err(PlayerError::AutomationDenied),
                2 => Err(PlayerError::Unavailable),
                _ => Err(PlayerError::MalformedResponse),
            };
        }

        // SAFETY: the native adapter guarantees NUL-terminated UTF-8.
        let decoded =
            serde_json::from_slice::<NowPlayingDto>(unsafe { CStr::from_ptr(json) }.to_bytes())
                .map_err(|_| PlayerError::MalformedResponse);
        // SAFETY: json was allocated by neosicht_now_playing.
        unsafe { neosicht_music_free_string(json) };
        let decoded = decoded?;

        Ok(Some(NowPlaying {
            application: match decoded.application {
                ApplicationDto::Spotify => MusicApplication::Spotify,
                ApplicationDto::AppleMusic => MusicApplication::AppleMusic,
            },
            state: match decoded.state {
                StateDto::Playing => PlaybackState::Playing,
                StateDto::Paused => PlaybackState::Paused,
            },
            title: decoded.title,
            artist: decoded.artist,
            artwork_url: (!decoded.artwork_url.is_empty()).then_some(decoded.artwork_url),
            embedded_artwork: decode_artwork(&decoded.artwork_data),
            position_seconds: decoded.position_seconds,
            duration_seconds: decoded.duration_seconds,
        }))
    }
}

impl PlayerSource for AppleScriptPlayerSource {
    fn now_playing(&self) -> Result<Option<NowPlaying>, PlayerError> {
        let snapshot = self.read(false)?;
        let track_key = snapshot
            .as_ref()
            .map(|track| format!("{:?}:{}:{}", track.application, track.title, track.artist));
        let changed = self
            .artwork_track
            .lock()
            .map_err(|_| PlayerError::Unavailable)?
            .as_ref()
            != track_key.as_ref();
        if changed {
            *self
                .artwork_track
                .lock()
                .map_err(|_| PlayerError::Unavailable)? = track_key;
            self.read(true)
        } else {
            Ok(snapshot)
        }
    }

    fn perform(
        &self,
        application: MusicApplication,
        action: TransportAction,
    ) -> Result<(), PlayerError> {
        let application = match application {
            MusicApplication::Spotify => 0,
            MusicApplication::AppleMusic => 1,
        };
        let action = match action {
            TransportAction::Previous => 0,
            TransportAction::TogglePlayback => 1,
            TransportAction::Next => 2,
        };
        // SAFETY: both integers are closed enum values understood by the shim.
        match unsafe { neosicht_music_transport(application, action) } {
            0 => Ok(()),
            1 => Err(PlayerError::AutomationDenied),
            _ => Err(PlayerError::Unavailable),
        }
    }
}

fn decode_artwork(encoded: &str) -> Option<Artwork> {
    if encoded.is_empty() {
        return None;
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .ok()?;
    let format = if bytes.starts_with(b"\x89PNG") {
        ArtworkFormat::Png
    } else if bytes.get(8..12).is_some_and(|header| header == b"WEBP") {
        ArtworkFormat::Webp
    } else if bytes.starts_with(b"\xff\xd8\xff") {
        ArtworkFormat::Jpeg
    } else {
        return None;
    };
    Some(Artwork { format, bytes })
}
