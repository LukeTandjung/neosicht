#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShellPanelBounds {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShellPanelPlacement {
    pub top_offset: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellPanelError {
    WindowUnavailable,
}

pub trait ShellPanel {
    fn place_above_system_menu_bar(
        &self,
        bounds: ShellPanelBounds,
    ) -> Result<ShellPanelPlacement, ShellPanelError>;
}
