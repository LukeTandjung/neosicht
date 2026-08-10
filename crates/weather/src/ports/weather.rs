use crate::core::forecast::{Location, WeatherReport};

#[derive(Clone, Debug, PartialEq)]
pub enum LocationError {
    PermissionDenied,
    Unavailable,
    InvalidCoordinates,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ForecastError {
    Unavailable,
    MalformedResponse,
}

/// Resolves the current physical position without exposing Core Location.
pub trait LocationSource: Send + Sync {
    fn current_location(&self) -> Result<Location, LocationError>;
}

/// Resolves weather for coordinates without exposing an HTTP provider's DTOs.
pub trait ForecastSource: Send + Sync {
    fn forecast(&self, location: Location) -> Result<WeatherReport, ForecastError>;
}
