use std::time::Duration;

use reqwest::blocking::Client;
use serde::Deserialize;

use crate::core::forecast::{DailyForecast, Location, WeatherReport};
use crate::ports::weather::{ForecastError, ForecastSource};

const FORECAST_URL: &str = "https://api.open-meteo.com/v1/forecast";

#[derive(Deserialize)]
struct ResponseDto {
    current: CurrentDto,
    daily: DailyDto,
}

#[derive(Deserialize)]
struct CurrentDto {
    temperature_2m: f32,
    apparent_temperature: f32,
    weather_code: u16,
}

#[derive(Deserialize)]
struct DailyDto {
    time: Vec<String>,
    weather_code: Vec<u16>,
    temperature_2m_max: Vec<f32>,
    temperature_2m_min: Vec<f32>,
}

/// Keyless Open-Meteo forecast adapter. Provider query parameters and response
/// arrays are normalized into one application weather report.
pub struct OpenMeteoForecastSource {
    client: Client,
}

impl OpenMeteoForecastSource {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl ForecastSource for OpenMeteoForecastSource {
    fn forecast(&self, location: Location) -> Result<WeatherReport, ForecastError> {
        let response = self
            .client
            .get(FORECAST_URL)
            .timeout(Duration::from_secs(10))
            .header("User-Agent", "neosicht/0.1")
            .query(&[
                ("latitude", location.coordinates.latitude.to_string()),
                ("longitude", location.coordinates.longitude.to_string()),
                (
                    "current",
                    "temperature_2m,apparent_temperature,weather_code".to_owned(),
                ),
                (
                    "daily",
                    "weather_code,temperature_2m_max,temperature_2m_min".to_owned(),
                ),
                ("forecast_days", "5".to_owned()),
                ("timezone", "auto".to_owned()),
            ])
            .send()
            .map_err(|_| ForecastError::Unavailable)?
            .error_for_status()
            .map_err(|_| ForecastError::Unavailable)?
            .json::<ResponseDto>()
            .map_err(|_| ForecastError::MalformedResponse)?;

        let day_count = response
            .daily
            .time
            .len()
            .min(response.daily.weather_code.len())
            .min(response.daily.temperature_2m_max.len())
            .min(response.daily.temperature_2m_min.len());
        if day_count == 0 {
            return Err(ForecastError::MalformedResponse);
        }

        let daily = (0..day_count)
            .map(|index| DailyForecast {
                date: response.daily.time[index].clone(),
                weather_code: response.daily.weather_code[index],
                high_celsius: response.daily.temperature_2m_max[index],
                low_celsius: response.daily.temperature_2m_min[index],
            })
            .collect();

        Ok(WeatherReport {
            place_name: location.place_name,
            weather_code: response.current.weather_code,
            temperature_celsius: response.current.temperature_2m,
            apparent_temperature_celsius: response.current.apparent_temperature,
            daily,
        })
    }
}
