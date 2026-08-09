use std::fmt;

use crate::core::menu::AppMenu;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuProviderError {
    /// The menu source is not available for this application.
    Unavailable(String),
    /// The menu source cannot perform this operation at all.
    Unsupported,
    /// The menu source ran but the operation failed.
    Failed(String),
}

impl fmt::Display for MenuProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(detail) => write!(f, "menu source unavailable: {detail}"),
            Self::Unsupported => f.write_str("menu source does not support this operation"),
            Self::Failed(detail) => write!(f, "menu operation failed: {detail}"),
        }
    }
}

/// The application-menu side of the bar: read the frontmost app's menu tree
/// and perform its items. The production adapter will be Accessibility-based;
/// until then a placeholder serves the standard macOS skeleton.
pub trait MenuProvider: Send + Sync {
    /// The menu tree of the named application, top-level titles in order.
    fn menus_for(&self, app_name: &str) -> Result<Vec<AppMenu>, MenuProviderError>;

    /// Perform a menu item as the application would (e.g. an AX press),
    /// addressed by its top-level menu title and stable entry position. Labels
    /// are display data and are not unique identifiers.
    fn activate(
        &self,
        app_name: &str,
        menu_title: &str,
        item_index: usize,
    ) -> Result<(), MenuProviderError>;
}
