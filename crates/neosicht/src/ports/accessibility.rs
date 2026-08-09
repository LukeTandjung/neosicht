#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccessibleApplication {
    pub pid: i32,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccessibleMenuItem {
    pub title: String,
    pub enabled: bool,
    pub shortcut: String,
    pub has_submenu: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AccessibilityError(pub i32);

pub trait Accessibility {
    fn request_permission(&self, prompt: bool) -> bool;
    fn frontmost_application(&self) -> Result<AccessibleApplication, AccessibilityError>;
    fn menu_titles(&self, pid: i32) -> Result<Vec<String>, AccessibilityError>;
    fn menu_items(
        &self,
        pid: i32,
        menu_title: &str,
    ) -> Result<Vec<AccessibleMenuItem>, AccessibilityError>;
    fn press_menu_path(&self, pid: i32, path: &[String]) -> Result<(), AccessibilityError>;
}
