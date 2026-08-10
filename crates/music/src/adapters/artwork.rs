use std::time::Duration;

use reqwest::blocking::Client;

use crate::core::playback::{Artwork, ArtworkFormat};
use crate::ports::player::{ArtworkSource, PlayerError};

const MAX_ARTWORK_BYTES: usize = 10 * 1024 * 1024;

pub struct HttpArtworkSource {
    client: Client,
}

impl HttpArtworkSource {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl ArtworkSource for HttpArtworkSource {
    fn load(&self, url: &str) -> Result<Artwork, PlayerError> {
        let response = self
            .client
            .get(url)
            .timeout(Duration::from_secs(10))
            .header("User-Agent", "neosicht/0.1")
            .send()
            .map_err(|_| PlayerError::Unavailable)?
            .error_for_status()
            .map_err(|_| PlayerError::Unavailable)?;
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_owned();
        let bytes = response
            .bytes()
            .map_err(|_| PlayerError::Unavailable)?
            .to_vec();
        if bytes.len() > MAX_ARTWORK_BYTES {
            return Err(PlayerError::MalformedResponse);
        }

        let format = if content_type.contains("png") || bytes.starts_with(b"\x89PNG") {
            ArtworkFormat::Png
        } else if content_type.contains("webp")
            || bytes.get(8..12).is_some_and(|header| header == b"WEBP")
        {
            ArtworkFormat::Webp
        } else if content_type.contains("jpeg") || bytes.starts_with(b"\xff\xd8\xff") {
            ArtworkFormat::Jpeg
        } else {
            return Err(PlayerError::MalformedResponse);
        };

        Ok(Artwork { format, bytes })
    }
}
