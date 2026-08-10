#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

impl Coordinates {
    pub fn new(latitude: f64, longitude: f64) -> Option<Self> {
        ((-90.0..=90.0).contains(&latitude) && (-180.0..=180.0).contains(&longitude)).then_some(
            Self {
                latitude,
                longitude,
            },
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Location {
    pub coordinates: Coordinates,
    pub place_name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DailyForecast {
    pub date: String,
    pub weather_code: u16,
    pub high_celsius: f32,
    pub low_celsius: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WeatherReport {
    pub place_name: String,
    pub weather_code: u16,
    pub temperature_celsius: f32,
    pub apparent_temperature_celsius: f32,
    pub daily: Vec<DailyForecast>,
}

pub fn condition_name(weather_code: u16) -> &'static str {
    match weather_code {
        0 => "Clear",
        1..=3 => "Partly cloudy",
        45 | 48 => "Fog",
        51..=57 => "Drizzle",
        61..=67 => "Rain",
        71..=77 => "Snow",
        80..=82 => "Rain showers",
        85 | 86 => "Snow showers",
        95..=99 => "Thunderstorm",
        _ => "Unknown",
    }
}
