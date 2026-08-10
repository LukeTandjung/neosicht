use std::ffi::{CStr, c_char, c_int};

use serde::Deserialize;

use crate::core::forecast::{Coordinates, Location};
use crate::ports::weather::{LocationError, LocationSource};

unsafe extern "C" {
    fn neosicht_current_location(error_code: *mut c_int) -> *mut c_char;
    fn neosicht_weather_free_string(value: *mut c_char);
}

#[derive(Deserialize)]
struct LocationDto {
    latitude: f64,
    longitude: f64,
    place_name: String,
}

/// Core Location reader whose native implementation owns authorization,
/// geocoding, and all Objective-C values.
pub struct CoreLocationSource;

impl LocationSource for CoreLocationSource {
    fn current_location(&self) -> Result<Location, LocationError> {
        let mut error_code = 0;
        // SAFETY: error_code is writable and a non-null result follows the
        // paired owned-string contract.
        let json = unsafe { neosicht_current_location(&mut error_code) };
        if json.is_null() {
            return Err(match error_code {
                1 => LocationError::PermissionDenied,
                2 => LocationError::Unavailable,
                _ => LocationError::InvalidCoordinates,
            });
        }

        // SAFETY: the native adapter returns NUL-terminated UTF-8.
        let decoded =
            serde_json::from_slice::<LocationDto>(unsafe { CStr::from_ptr(json) }.to_bytes())
                .map_err(|_| LocationError::InvalidCoordinates);
        // SAFETY: json was allocated by neosicht_current_location.
        unsafe { neosicht_weather_free_string(json) };

        let decoded = decoded?;
        let coordinates = Coordinates::new(decoded.latitude, decoded.longitude)
            .ok_or(LocationError::InvalidCoordinates)?;
        Ok(Location {
            coordinates,
            place_name: decoded.place_name,
        })
    }
}
