use crate::core::forecast::WeatherReport;
use crate::ports::weather::{ForecastError, ForecastSource, LocationError, LocationSource};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub enum WeatherLoadError {
    Location(LocationError),
    Forecast(ForecastError),
}

/// Owns location-to-forecast orchestration and refresh policy. Consumers need
/// no knowledge of Core Location, Open-Meteo, or their error contracts.
pub struct WeatherService {
    locations: Arc<dyn LocationSource>,
    forecasts: Arc<dyn ForecastSource>,
}

impl WeatherService {
    pub fn new(locations: Arc<dyn LocationSource>, forecasts: Arc<dyn ForecastSource>) -> Self {
        Self {
            locations,
            forecasts,
        }
    }

    pub fn load(&self) -> Result<WeatherReport, WeatherLoadError> {
        let location = self
            .locations
            .current_location()
            .map_err(WeatherLoadError::Location)?;
        self.forecasts
            .forecast(location)
            .map_err(WeatherLoadError::Forecast)
    }
}
