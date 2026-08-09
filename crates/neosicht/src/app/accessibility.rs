use crate::ports::accessibility::{
    Accessibility, AccessibilityError, AccessibleApplication, AccessibleMenuItem,
};

pub struct AccessibilityApp<A> {
    accessibility: A,
}

impl<A> AccessibilityApp<A>
where
    A: Accessibility,
{
    pub fn new(accessibility: A) -> Self {
        Self { accessibility }
    }

    pub fn request_permission(&self, prompt: bool) -> bool {
        self.accessibility.request_permission(prompt)
    }

    pub fn frontmost_application(&self) -> Result<AccessibleApplication, AccessibilityError> {
        self.accessibility.frontmost_application()
    }

    pub fn menu_titles(&self, pid: i32) -> Result<Vec<String>, AccessibilityError> {
        self.accessibility.menu_titles(pid)
    }

    pub fn menu_items(
        &self,
        pid: i32,
        menu_title: &str,
    ) -> Result<Vec<AccessibleMenuItem>, AccessibilityError> {
        self.accessibility.menu_items(pid, menu_title)
    }

    pub fn press_menu_path(&self, pid: i32, path: &[String]) -> Result<(), AccessibilityError> {
        self.accessibility.press_menu_path(pid, path)
    }
}
