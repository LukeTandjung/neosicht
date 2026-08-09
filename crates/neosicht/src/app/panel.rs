use crate::ports::panel::{
    ShellPanel, ShellPanelBounds, ShellPanelError, ShellPanelInteraction, ShellPanelPlacement,
};

/// Application boundary for the shell panel. It owns the policy that the
/// permanent transparent panel accepts input only over the bar unless popup
/// content is visible.
pub struct ShellPanelService<P> {
    panel: P,
    bar_height: f64,
}

impl<P: ShellPanel> ShellPanelService<P> {
    pub fn new(panel: P, bar_height: f64) -> Self {
        Self { panel, bar_height }
    }

    pub fn initialize(
        &self,
        bounds: ShellPanelBounds,
    ) -> Result<ShellPanelPlacement, ShellPanelError> {
        let placement = self.panel.place_above_system_menu_bar(bounds)?;
        self.panel.set_interaction(ShellPanelInteraction::BarOnly {
            bar_height: self.bar_height,
        })?;
        Ok(placement)
    }

    pub fn set_popup_visible(&self, visible: bool) -> Result<(), ShellPanelError> {
        let interaction = if visible {
            ShellPanelInteraction::Extended
        } else {
            ShellPanelInteraction::BarOnly {
                bar_height: self.bar_height,
            }
        };
        self.panel.set_interaction(interaction)
    }
}
