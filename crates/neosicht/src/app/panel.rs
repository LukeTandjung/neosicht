use crate::ports::panel::{ShellPanel, ShellPanelBounds, ShellPanelError, ShellPanelPlacement};

pub fn place_shell_panel(
    panel: &impl ShellPanel,
    bounds: ShellPanelBounds,
) -> Result<ShellPanelPlacement, ShellPanelError> {
    panel.place_above_system_menu_bar(bounds)
}
