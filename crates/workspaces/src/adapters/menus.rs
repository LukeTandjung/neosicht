use std::ffi::{CStr, CString, c_char, c_void};
use std::sync::Mutex;

use crate::core::menu::{AppMenu, MenuEntry, MenuItemSpec};
use crate::ports::menus::{MenuProvider, MenuProviderError};

pub struct AccessibilityMenuProvider {
    cached: Mutex<Option<(String, Vec<AppMenu>)>>,
}

impl AccessibilityMenuProvider {
    pub fn new() -> Self {
        Self {
            cached: Mutex::new(None),
        }
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn neosicht_copy_frontmost_menus(
        app_name: *const c_char,
        context: *mut c_void,
        menu_callback: unsafe extern "C" fn(*mut c_void, *const c_char),
        separator_callback: unsafe extern "C" fn(*mut c_void),
        item_callback: unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char, bool, bool),
    ) -> bool;
    fn neosicht_activate_frontmost_menu_item(
        app_name: *const c_char,
        menu_title: *const c_char,
        item_index: usize,
    ) -> bool;
}

#[cfg(target_os = "macos")]
unsafe fn text(value: *const c_char) -> String {
    if value.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(value) }
        .to_string_lossy()
        .into_owned()
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn add_menu(context: *mut c_void, title: *const c_char) {
    let menus = unsafe { &mut *(context.cast::<Vec<AppMenu>>()) };
    menus.push(AppMenu {
        title: unsafe { text(title) },
        entries: Vec::new(),
    });
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn add_separator(context: *mut c_void) {
    let menus = unsafe { &mut *(context.cast::<Vec<AppMenu>>()) };
    if let Some(menu) = menus.last_mut() {
        menu.entries.push(MenuEntry::Separator);
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn add_item(
    context: *mut c_void,
    label: *const c_char,
    shortcut: *const c_char,
    checked: bool,
    enabled: bool,
) {
    let menus = unsafe { &mut *(context.cast::<Vec<AppMenu>>()) };
    let Some(menu) = menus.last_mut() else {
        return;
    };
    let shortcut = unsafe { text(shortcut) };
    menu.entries.push(MenuEntry::Item(MenuItemSpec {
        label: unsafe { text(label) },
        shortcut: (!shortcut.is_empty()).then_some(shortcut),
        checked,
        enabled,
    }));
}

impl MenuProvider for AccessibilityMenuProvider {
    fn menus_for(&self, app_name: &str) -> Result<Vec<AppMenu>, MenuProviderError> {
        #[cfg(target_os = "macos")]
        {
            let mut cached = self
                .cached
                .lock()
                .map_err(|_| MenuProviderError::Failed("menu cache lock poisoned".to_owned()))?;
            if let Some((cached_app, menus)) = cached.as_ref()
                && cached_app == app_name
            {
                return Ok(menus.clone());
            }

            let native_app_name = CString::new(app_name).map_err(|_| {
                MenuProviderError::Failed("application name contains NUL".to_owned())
            })?;
            let mut menus: Vec<AppMenu> = Vec::new();
            let copied = unsafe {
                neosicht_copy_frontmost_menus(
                    native_app_name.as_ptr(),
                    (&mut menus as *mut Vec<AppMenu>).cast(),
                    add_menu,
                    add_separator,
                    add_item,
                )
            };
            if copied {
                menus.retain(|menu| {
                    menu.entries
                        .iter()
                        .any(|entry| matches!(entry, MenuEntry::Item(_)))
                });
                *cached = Some((app_name.to_owned(), menus.clone()));
                Ok(menus)
            } else {
                Err(MenuProviderError::Unavailable(
                    "Accessibility permission or the frontmost menu bar is unavailable".to_owned(),
                ))
            }
        }
        #[cfg(not(target_os = "macos"))]
        Err(MenuProviderError::Unsupported)
    }

    fn activate(
        &self,
        app_name: &str,
        menu_title: &str,
        item_index: usize,
    ) -> Result<(), MenuProviderError> {
        #[cfg(target_os = "macos")]
        {
            let app_name = CString::new(app_name).map_err(|_| {
                MenuProviderError::Failed("application name contains NUL".to_owned())
            })?;
            let menu_title = CString::new(menu_title)
                .map_err(|_| MenuProviderError::Failed("menu title contains NUL".to_owned()))?;
            if unsafe {
                neosicht_activate_frontmost_menu_item(
                    app_name.as_ptr(),
                    menu_title.as_ptr(),
                    item_index,
                )
            } {
                Ok(())
            } else {
                Err(MenuProviderError::Failed(format!(
                    "could not activate {menu_title:?} entry {item_index}"
                )))
            }
        }
        #[cfg(not(target_os = "macos"))]
        Err(MenuProviderError::Unsupported)
    }
}
