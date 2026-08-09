//! Accessibility adapter for application menu discovery and actions.

use core_foundation::array::CFArray;
use core_foundation::base::{CFType, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
use core_foundation::number::CFNumber;
use core_foundation::string::{CFString, CFStringRef};
use std::ffi::c_void;

use crate::ports::accessibility::{
    Accessibility, AccessibilityError, AccessibleApplication, AccessibleMenuItem,
};

// ---------------------------------------------------------------------------
// Raw FFI. AXUIElement is a plain C API in ApplicationServices — exactly the
// boundary used by this adapter.
// ---------------------------------------------------------------------------

/// AX elements are CoreFoundation objects; we carry them as `CFType` for RAII
/// (release on drop, retain on clone) and pass the raw pointer across FFI.
type AXUIElementRef = *const c_void;
type AXError = i32;

const AX_SUCCESS: AXError = 0;

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> u8;
    fn AXUIElementCreateSystemWide() -> AXUIElementRef;
    fn AXUIElementCreateApplication(pid: i32) -> AXUIElementRef;
    fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: *mut *const c_void,
    ) -> AXError;
    fn AXUIElementPerformAction(element: AXUIElementRef, action: CFStringRef) -> AXError;
    fn AXUIElementSetMessagingTimeout(element: AXUIElementRef, timeout_secs: f32) -> AXError;
    fn AXUIElementGetPid(element: AXUIElementRef, pid: *mut i32) -> AXError;
}

// ---------------------------------------------------------------------------
// Thin safe helpers over the raw calls
// ---------------------------------------------------------------------------

fn attr(element: &CFType, name: &str) -> Option<CFType> {
    let key = CFString::new(name);
    let mut out: *const c_void = std::ptr::null();
    let err = unsafe {
        AXUIElementCopyAttributeValue(element.as_CFTypeRef(), key.as_concrete_TypeRef(), &mut out)
    };
    if err == AX_SUCCESS && !out.is_null() {
        // "Copy" in the name → create rule: we own one reference.
        Some(unsafe { CFType::wrap_under_create_rule(out) })
    } else {
        None
    }
}

fn string_attr(element: &CFType, name: &str) -> Option<String> {
    attr(element, name)?
        .downcast_into::<CFString>()
        .map(|s| s.to_string())
}

fn bool_attr(element: &CFType, name: &str) -> Option<bool> {
    attr(element, name)?
        .downcast_into::<CFBoolean>()
        .map(|b| b == CFBoolean::true_value())
}

fn children(element: &CFType) -> Vec<CFType> {
    attr(element, "AXChildren")
        .and_then(|value| value.downcast_into::<CFArray>())
        .map(|array| {
            array
                .iter()
                // Items are borrowed from the array (get rule → retain).
                .map(|item| unsafe { CFType::wrap_under_get_rule(*item) })
                .collect()
        })
        .unwrap_or_default()
}

fn press(element: &CFType) -> AXError {
    let action = CFString::new("AXPress");
    unsafe { AXUIElementPerformAction(element.as_CFTypeRef(), action.as_concrete_TypeRef()) }
}

fn application_element(pid: i32) -> CFType {
    let raw = unsafe { AXUIElementCreateApplication(pid) };
    let element = unsafe { CFType::wrap_under_create_rule(raw) };
    // Hung-app safety: never let one synchronous AX call block us for long.
    unsafe { AXUIElementSetMessagingTimeout(element.as_CFTypeRef(), 1.0) };
    element
}

fn frontmost_application() -> Option<(i32, CFType)> {
    let system_wide = unsafe { CFType::wrap_under_create_rule(AXUIElementCreateSystemWide()) };
    let key = CFString::new("AXFocusedApplication");
    let mut raw: *const c_void = std::ptr::null();
    let err = unsafe {
        AXUIElementCopyAttributeValue(
            system_wide.as_CFTypeRef(),
            key.as_concrete_TypeRef(),
            &mut raw,
        )
    };
    if err != AX_SUCCESS {
        eprintln!("AXFocusedApplication failed: AXError {err}");
        return None;
    }
    let focused = unsafe { CFType::wrap_under_create_rule(raw) };
    let mut pid: i32 = 0;
    let err = unsafe { AXUIElementGetPid(focused.as_CFTypeRef(), &mut pid) };
    (err == AX_SUCCESS).then(|| (pid, application_element(pid)))
}

/// Menu containers interpose a single `AXMenu` child between an item and its
/// entries; descend through it transparently.
fn menu_entries(element: &CFType) -> Vec<CFType> {
    let direct = children(element);
    match direct.as_slice() {
        [only] if string_attr(only, "AXRole").as_deref() == Some("AXMenu") => children(only),
        _ => direct,
    }
}

fn shortcut_of(item: &CFType) -> String {
    let cmd_char = string_attr(item, "AXMenuItemCmdChar").unwrap_or_default();
    if cmd_char.is_empty() {
        return String::new();
    }
    let modifiers = attr(item, "AXMenuItemCmdModifiers")
        .and_then(|value| value.downcast_into::<CFNumber>())
        .and_then(|number| number.to_i64())
        .unwrap_or(0);
    // Bitmask per HIToolbox: 1 = shift, 2 = option, 4 = control, 8 = no cmd.
    let mut out = String::new();
    if modifiers & 4 != 0 {
        out.push('⌃');
    }
    if modifiers & 2 != 0 {
        out.push('⌥');
    }
    if modifiers & 1 != 0 {
        out.push('⇧');
    }
    if modifiers & 8 == 0 {
        out.push('⌘');
    }
    out.push_str(&cmd_char);
    out
}

fn find_titled(container: &CFType, title: &str) -> Option<CFType> {
    menu_entries(container)
        .into_iter()
        .find(|entry| string_attr(entry, "AXTitle").as_deref() == Some(title))
}

#[derive(Clone, Copy, Default)]
pub struct AxAccessibility;

impl Accessibility for AxAccessibility {
    fn request_permission(&self, prompt: bool) -> bool {
        let key = CFString::new("AXTrustedCheckOptionPrompt");
        let prompt = if prompt {
            CFBoolean::true_value()
        } else {
            CFBoolean::false_value()
        };
        let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), prompt.as_CFType())]);
        (unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) }) != 0
    }

    fn frontmost_application(&self) -> Result<AccessibleApplication, AccessibilityError> {
        let (pid, app) = frontmost_application().ok_or(AccessibilityError(-1))?;
        let title = string_attr(&app, "AXTitle").ok_or(AccessibilityError(-1))?;
        Ok(AccessibleApplication { pid, title })
    }

    fn menu_titles(&self, pid: i32) -> Result<Vec<String>, AccessibilityError> {
        let app = application_element(pid);
        let menu_bar = attr(&app, "AXMenuBar").ok_or(AccessibilityError(-1))?;
        Ok(children(&menu_bar)
            .iter()
            .filter_map(|item| string_attr(item, "AXTitle"))
            .collect())
    }

    fn menu_items(
        &self,
        pid: i32,
        menu_title: &str,
    ) -> Result<Vec<AccessibleMenuItem>, AccessibilityError> {
        let app = application_element(pid);
        let menu_bar = attr(&app, "AXMenuBar").ok_or(AccessibilityError(-1))?;
        let menu = find_titled(&menu_bar, menu_title).ok_or(AccessibilityError(-1))?;
        Ok(menu_entries(&menu)
            .iter()
            .map(|item| AccessibleMenuItem {
                title: string_attr(item, "AXTitle").unwrap_or_default(),
                enabled: bool_attr(item, "AXEnabled").unwrap_or(false),
                shortcut: shortcut_of(item),
                has_submenu: !menu_entries(item).is_empty(),
            })
            .collect())
    }

    fn press_menu_path(&self, pid: i32, path: &[String]) -> Result<(), AccessibilityError> {
        let app = application_element(pid);
        let mut current = attr(&app, "AXMenuBar").ok_or(AccessibilityError(-1))?;
        for component in path {
            current = find_titled(&current, component).ok_or(AccessibilityError(-1))?;
        }
        let error = press(&current);
        if error == AX_SUCCESS {
            Ok(())
        } else {
            Err(AccessibilityError(error))
        }
    }
}
