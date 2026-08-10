//! Live Spotify and Apple Music presentation and transport controls.

use std::sync::Arc;
use std::time::Duration;

use base_gpui::button::ButtonRoot;
use base_gpui::progress::{ProgressIndicator, ProgressRoot, ProgressTrack};
use gpui::{
    Animation, AnimationExt as _, AnyElement, Context, EventEmitter, FontWeight, Image,
    ImageFormat, Task, WeakEntity, Window, bounce, div, ease_in_out, img, linear_color_stop,
    linear_gradient, prelude::*, px, relative,
};
use theme::core::{palette, typography};

use crate::app::player::PlayerService;
use crate::core::playback::{ArtworkFormat, NowPlaying, PlaybackState, TransportAction};
use crate::ports::player::PlayerError;

pub enum SectionEvent {
    Changed,
}

pub struct MusicSection {
    service: Arc<PlayerService>,
    now_playing: Option<NowPlaying>,
    artwork_url: Option<String>,
    artwork_key: Option<String>,
    artwork: Option<Arc<Image>>,
    error: Option<PlayerError>,
    observer: Option<Task<()>>,
    operations: Vec<Task<()>>,
}

impl EventEmitter<SectionEvent> for MusicSection {}

impl MusicSection {
    pub fn new(service: Arc<PlayerService>) -> Self {
        Self {
            service,
            now_playing: None,
            artwork_url: None,
            artwork_key: None,
            artwork: None,
            error: None,
            observer: None,
            operations: Vec::new(),
        }
    }

    pub(crate) fn own_observer(&mut self, observer: Task<()>) {
        self.observer = Some(observer);
    }

    pub(crate) fn apply(
        &mut self,
        observed: Result<Option<NowPlaying>, PlayerError>,
        cx: &mut Context<Self>,
    ) {
        let previous = self.now_playing.clone();
        let previous_error = self.error.clone();
        match observed {
            Ok(now_playing) => {
                let artwork_key = now_playing.as_ref().map(|track| {
                    format!("{:?}:{}:{}", track.application, track.title, track.artist)
                });
                let embedded = now_playing
                    .as_ref()
                    .and_then(|track| track.embedded_artwork.clone());
                let artwork_url = embedded
                    .is_none()
                    .then(|| {
                        now_playing
                            .as_ref()
                            .and_then(|track| track.artwork_url.clone())
                    })
                    .flatten();
                if self.artwork_key != artwork_key {
                    self.artwork_key = artwork_key;
                    self.artwork_url = artwork_url.clone();
                    self.artwork = embedded.map(image_from_artwork);
                    if self.artwork.is_none()
                        && let Some(url) = artwork_url
                    {
                        self.fetch_artwork(url, cx);
                    }
                }
                self.now_playing = now_playing;
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
        if self.now_playing != previous || self.error != previous_error {
            cx.emit(SectionEvent::Changed);
        }
    }

    pub fn bar_fragment(&self) -> AnyElement {
        let playing = self
            .now_playing
            .as_ref()
            .is_some_and(|track| track.state == PlaybackState::Playing);
        let durations = [420, 510, 360];

        div()
            .flex()
            .items_end()
            .gap(px(2.))
            .h(px(11.))
            .flex_none()
            .children(durations.into_iter().enumerate().map(|(index, duration)| {
                let level = div().w(px(2.)).h(px(3.)).bg(palette::green());
                if playing {
                    level
                        .with_animation(
                            format!("music-level-{index}"),
                            Animation::new(Duration::from_millis(duration))
                                .repeat()
                                .with_easing(bounce(ease_in_out)),
                            |level, progress| level.h(px(3.0 + progress * 8.0)),
                        )
                        .into_any_element()
                } else {
                    level.into_any_element()
                }
            }))
            .into_any_element()
    }

    pub fn panel_column(&self, entity: WeakEntity<Self>) -> AnyElement {
        let Some(track) = &self.now_playing else {
            let message = match self.error {
                Some(PlayerError::AutomationDenied) => "Music automation access is required",
                Some(_) => "Music player unavailable",
                None => "Nothing playing",
            };
            return column_shell()
                .child(status_message(message))
                .into_any_element();
        };

        let artwork = self.artwork.clone();
        let position = format_duration(track.position_seconds);
        let duration = format_duration(track.duration_seconds);
        let toggle_glyph = if track.state == PlaybackState::Playing {
            "❚❚"
        } else {
            "▶"
        };

        column_shell()
            .child(
                div()
                    .w_full()
                    .aspect_square()
                    .rounded(px(8.))
                    .mt(px(8.))
                    .border_1()
                    .border_color(palette::border())
                    .overflow_hidden()
                    .bg(linear_gradient(
                        135.,
                        linear_color_stop(palette::raise(), 0.),
                        linear_color_stop(palette::bar(), 1.),
                    ))
                    .when_some(artwork, |cover, artwork| {
                        cover.child(img(artwork).size_full().object_fit(gpui::ObjectFit::Cover))
                    }),
            )
            .child(
                div()
                    .text_size(px(12.5))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(palette::text_bright())
                    .mt(px(8.))
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .child(track.title.clone()),
            )
            .child(
                div()
                    .font_family(typography::mono())
                    .text_size(px(10.5))
                    .text_color(palette::subtle())
                    .mt(px(2.))
                    .child(track.artist.clone()),
            )
            .child(
                ProgressRoot::new()
                    .id("music-progress")
                    .value(Some(track.progress_percentage()))
                    .label("Track position")
                    .mt(px(10.))
                    .child(
                        ProgressTrack::new()
                            .w_full()
                            .h(px(3.))
                            .rounded(px(2.))
                            .bg(palette::raise())
                            .overflow_hidden()
                            .child(
                                ProgressIndicator::new()
                                    .h_full()
                                    .bg(palette::accent())
                                    .style_with_state(|state, indicator| {
                                        let fraction = state.percentage.unwrap_or(0.0) / 100.0;
                                        indicator.w(relative(fraction as f32))
                                    }),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .justify_between()
                    .mt(px(4.))
                    .font_family(typography::mono())
                    .text_size(px(9.5))
                    .text_color(palette::muted())
                    .child(position)
                    .child(duration),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap(px(8.))
                    .mt(px(10.))
                    .child(transport_button(
                        "music-prev",
                        "«",
                        TransportAction::Previous,
                        entity.clone(),
                    ))
                    .child(
                        transport_button(
                            "music-toggle",
                            toggle_glyph,
                            TransportAction::TogglePlayback,
                            entity.clone(),
                        )
                        .size(px(30.))
                        .bg(palette::accent())
                        .text_color(palette::ink()),
                    )
                    .child(transport_button(
                        "music-next",
                        "»",
                        TransportAction::Next,
                        entity,
                    )),
            )
            .into_any_element()
    }

    fn fetch_artwork(&mut self, url: String, cx: &mut Context<Self>) {
        let service = self.service.clone();
        let operation = cx.spawn(async move |this, cx| {
            let expected_url = url.clone();
            let loaded = cx
                .background_executor()
                .spawn(async move { service.load_artwork(&url) })
                .await;
            if let Ok(artwork) = loaded {
                this.update(cx, |section, cx| {
                    if section.artwork_url.as_deref() == Some(expected_url.as_str()) {
                        section.artwork = Some(image_from_artwork(artwork));
                        cx.emit(SectionEvent::Changed);
                    }
                })
                .ok();
            }
        });
        self.operations.push(operation);
    }

    fn perform(&mut self, action: TransportAction, cx: &mut Context<Self>) {
        let Some(application) = self.now_playing.as_ref().map(|track| track.application) else {
            return;
        };
        let service = self.service.clone();
        let operation = cx.spawn(async move |this, cx| {
            let performed = cx
                .background_executor()
                .spawn(async move { service.perform(application, action) })
                .await;
            if let Err(error) = performed {
                this.update(cx, |section, cx| {
                    if section.error.as_ref() != Some(&error) {
                        section.error = Some(error);
                        cx.emit(SectionEvent::Changed);
                    }
                })
                .ok();
            }
        });
        self.operations.push(operation);
    }
}

fn column_shell() -> gpui::Div {
    div().flex_1().min_w_0().child(
        div()
            .font_family(typography::mono())
            .text_size(px(9.5))
            .text_color(palette::muted())
            .child("NOW PLAYING"),
    )
}

fn status_message(message: &'static str) -> gpui::Div {
    div()
        .mt(px(10.))
        .font_family(typography::mono())
        .text_size(px(10.))
        .text_color(palette::muted())
        .child(message)
}

fn transport_button(
    id: &'static str,
    glyph: &'static str,
    action: TransportAction,
    entity: WeakEntity<MusicSection>,
) -> ButtonRoot {
    ButtonRoot::new()
        .id(id)
        .aria_label(id)
        .size(px(26.))
        .rounded(px(6.))
        .border_1()
        .border_color(palette::border())
        .flex()
        .items_center()
        .justify_center()
        .font_family(typography::mono())
        .text_size(px(10.))
        .text_color(palette::text())
        .style_with_state(|_state, button| button.hover(|style| style.bg(palette::raise())))
        .on_click(move |_, _window: &mut Window, cx| {
            entity
                .update(cx, |section, cx| section.perform(action, cx))
                .ok();
        })
        .child(glyph)
}

fn image_from_artwork(artwork: crate::core::playback::Artwork) -> Arc<Image> {
    let format = match artwork.format {
        ArtworkFormat::Jpeg => ImageFormat::Jpeg,
        ArtworkFormat::Png => ImageFormat::Png,
        ArtworkFormat::Webp => ImageFormat::Webp,
    };
    Arc::new(Image::from_bytes(format, artwork.bytes))
}

fn format_duration(seconds: f64) -> String {
    let seconds = seconds.max(0.0).round() as u64;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}
