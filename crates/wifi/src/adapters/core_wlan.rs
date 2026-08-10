use std::ffi::{CStr, CString, c_char};

use serde::Deserialize;

use crate::core::network::{WifiNetwork, WifiSnapshot};
use crate::ports::wifi::{WifiError, WifiProvider};

unsafe extern "C" {
    fn neosicht_wifi_snapshot() -> *mut c_char;
    fn neosicht_wifi_set_enabled(enabled: i32) -> i32;
    fn neosicht_wifi_join(ssid: *const c_char, password: *const c_char, remember: i32) -> i32;
    fn free(pointer: *mut std::ffi::c_void);
}

#[derive(Deserialize)]
struct NativeSnapshot {
    enabled: bool,
    connected: Option<String>,
    networks: Vec<NativeNetwork>,
}

#[derive(Deserialize)]
struct NativeNetwork {
    ssid: String,
    signal: i32,
    secure: bool,
    known: bool,
}

pub struct CoreWlanProvider;

impl WifiProvider for CoreWlanProvider {
    fn observe(&self) -> Result<WifiSnapshot, WifiError> {
        let pointer = unsafe { neosicht_wifi_snapshot() };
        if pointer.is_null() {
            return Err(WifiError::Unavailable(
                "CoreWLAN has no interface".to_owned(),
            ));
        }
        let json = unsafe { CStr::from_ptr(pointer) }
            .to_string_lossy()
            .into_owned();
        unsafe { free(pointer.cast()) };
        let native: NativeSnapshot =
            serde_json::from_str(&json).map_err(|error| WifiError::Malformed(error.to_string()))?;
        Ok(WifiSnapshot {
            enabled: native.enabled,
            connected_ssid: native.connected,
            networks: native
                .networks
                .into_iter()
                .map(|network| WifiNetwork {
                    ssid: network.ssid,
                    signal: network.signal,
                    secure: network.secure,
                    known: network.known,
                })
                .collect(),
        })
    }

    fn set_enabled(&self, enabled: bool) -> Result<(), WifiError> {
        (unsafe { neosicht_wifi_set_enabled(i32::from(enabled)) } == 1)
            .then_some(())
            .ok_or_else(|| WifiError::Failed("failed to change Wi-Fi power".to_owned()))
    }

    fn join(&self, ssid: &str, password: Option<&str>, remember: bool) -> Result<(), WifiError> {
        let ssid = CString::new(ssid).map_err(|error| WifiError::Malformed(error.to_string()))?;
        let password = password
            .map(CString::new)
            .transpose()
            .map_err(|error| WifiError::Malformed(error.to_string()))?;
        let password_pointer = password
            .as_ref()
            .map_or(std::ptr::null(), |value| value.as_ptr());
        (unsafe { neosicht_wifi_join(ssid.as_ptr(), password_pointer, i32::from(remember)) } == 1)
            .then_some(())
            .ok_or_else(|| WifiError::Failed("failed to join Wi-Fi network".to_owned()))
    }
}
