use std::sync::Arc;

use weather::app::weather::WeatherService;
use weather::core::forecast::{Coordinates, Location, WeatherReport};
use weather::ports::weather::{ForecastError, ForecastSource, LocationError, LocationSource};

struct FixedLocation(Location);

impl LocationSource for FixedLocation {
    fn current_location(&self) -> Result<Location, LocationError> {
        Ok(self.0.clone())
    }
}

struct RecordingForecast;

impl ForecastSource for RecordingForecast {
    fn forecast(&self, location: Location) -> Result<WeatherReport, ForecastError> {
        Ok(WeatherReport {
            place_name: location.place_name,
            weather_code: 0,
            temperature_celsius: 21.0,
            apparent_temperature_celsius: 20.0,
            daily: Vec::new(),
        })
    }
}

#[test]
fn load_resolves_location_before_requesting_the_forecast() {
    let coordinates = Coordinates::new(40.7, -74.0).unwrap();
    let service = WeatherService::new(
        Arc::new(FixedLocation(Location {
            coordinates,
            place_name: "Brooklyn".to_owned(),
        })),
        Arc::new(RecordingForecast),
    );

    assert_eq!(service.load().unwrap().place_name, "Brooklyn");
}
