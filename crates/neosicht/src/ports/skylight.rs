pub type DisplayId = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkyLightError(pub i32);

pub trait SkyLight {
    fn active_displays(&self) -> Vec<DisplayId>;
    fn visibility_override(&self, display: DisplayId) -> Result<i32, SkyLightError>;
    fn set_visibility_override(&self, display: DisplayId, value: i32) -> Result<(), SkyLightError>;
    fn autohide(&self) -> Result<bool, SkyLightError>;
    fn set_autohide(&self, enabled: bool) -> Result<(), SkyLightError>;
    fn set_override_alpha(&self, alpha: f64) -> Result<(), SkyLightError>;
    fn reset_override_alpha(&self) -> Result<(), SkyLightError>;
    fn set_maximum_reveal(&self, reveal: f64) -> Result<(), SkyLightError>;
}
