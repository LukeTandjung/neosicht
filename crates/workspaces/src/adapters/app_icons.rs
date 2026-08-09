//! AppKit-backed icon source: NSWorkspace's icon for the running application,
//! resolved by bundle id or name in the Obj-C shim (`app_icons.m`), returned
//! as PNG bytes and cached per application.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::ports::icons::AppIconProvider;

/// Icons render at 15pt in the bar; 64px keeps them crisp on Retina.
const ICON_PIXEL_SIZE: i32 = 64;

#[derive(Default)]
pub struct AppKitIconProvider {
    cache: Mutex<HashMap<String, Arc<Vec<u8>>>>,
}

impl AppKitIconProvider {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AppIconProvider for AppKitIconProvider {
    fn icon_png(&self, bundle_id: Option<&str>, app_name: &str) -> Option<Arc<Vec<u8>>> {
        let key = bundle_id.unwrap_or(app_name).to_owned();
        if let Some(cached) = self.cache.lock().unwrap().get(&key) {
            return Some(cached.clone());
        }

        // Misses are not cached: an application that could not be resolved
        // this tick (e.g. name mismatch during launch) is retried next poll.
        let png = copy_app_icon_png(bundle_id, app_name)?;
        let png = Arc::new(png);
        self.cache.lock().unwrap().insert(key, png.clone());
        Some(png)
    }
}

#[cfg(target_os = "macos")]
fn copy_app_icon_png(bundle_id: Option<&str>, app_name: &str) -> Option<Vec<u8>> {
    use std::ffi::CString;

    #[link(name = "workspaces_native")]
    unsafe extern "C" {
        fn workspaces_copy_app_icon_png(
            bundle_id: *const std::ffi::c_char,
            app_name: *const std::ffi::c_char,
            pixel_size: i32,
            out_bytes: *mut *mut u8,
        ) -> i64;
        fn workspaces_free_icon_png(bytes: *mut u8);
    }

    let bundle_id = CString::new(bundle_id.unwrap_or_default()).ok()?;
    let app_name = CString::new(app_name).ok()?;
    let mut bytes: *mut u8 = std::ptr::null_mut();
    let length = unsafe {
        workspaces_copy_app_icon_png(
            bundle_id.as_ptr(),
            app_name.as_ptr(),
            ICON_PIXEL_SIZE,
            &mut bytes,
        )
    };
    if length <= 0 || bytes.is_null() {
        return None;
    }
    let png = unsafe {
        let slice = std::slice::from_raw_parts(bytes, length as usize);
        let owned = slice.to_vec();
        workspaces_free_icon_png(bytes);
        owned
    };
    Some(png)
}

#[cfg(not(target_os = "macos"))]
fn copy_app_icon_png(_bundle_id: Option<&str>, _app_name: &str) -> Option<Vec<u8>> {
    None
}
