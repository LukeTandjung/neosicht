use crate::ports::skylight::{DisplayId, SkyLight, SkyLightError};

pub struct SkyLightApp<S> {
    skylight: S,
}

impl<S> SkyLightApp<S>
where
    S: SkyLight,
{
    pub fn new(skylight: S) -> Self {
        Self { skylight }
    }

    pub fn active_displays(&self) -> Vec<DisplayId> {
        self.skylight.active_displays()
    }

    pub fn visibility_override(&self, display: DisplayId) -> Result<i32, SkyLightError> {
        self.skylight.visibility_override(display)
    }

    pub fn set_visibility_override(
        &self,
        display: DisplayId,
        value: i32,
    ) -> Result<(), SkyLightError> {
        self.skylight.set_visibility_override(display, value)
    }

    pub fn autohide(&self) -> Result<bool, SkyLightError> {
        self.skylight.autohide()
    }

    pub fn set_autohide(&self, enabled: bool) -> Result<(), SkyLightError> {
        self.skylight.set_autohide(enabled)
    }

    pub fn set_override_alpha(&self, alpha: f64) -> Result<(), SkyLightError> {
        self.skylight.set_override_alpha(alpha)
    }

    pub fn reset_override_alpha(&self) -> Result<(), SkyLightError> {
        self.skylight.reset_override_alpha()
    }

    pub fn set_maximum_reveal(&self, reveal: f64) -> Result<(), SkyLightError> {
        self.skylight.set_maximum_reveal(reveal)
    }
}
