/// One top-level menu of an application: its rail title and its items.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppMenu {
    pub title: String,
    pub entries: Vec<MenuEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuEntry {
    Separator,
    Item(MenuItemSpec),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuItemSpec {
    pub label: String,
    pub shortcut: Option<String>,
    pub checked: bool,
    pub enabled: bool,
}

impl MenuItemSpec {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            shortcut: None,
            checked: false,
            enabled: true,
        }
    }

    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }
}
