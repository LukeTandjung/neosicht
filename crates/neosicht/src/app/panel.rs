use crate::ports::panel::{ShellPanel, ShellPanelBounds, ShellPanelError, ShellPanelPlacement};

pub fn place_shell_panel(
    panel: &impl ShellPanel,
    bounds: ShellPanelBounds,
) -> Result<ShellPanelPlacement, ShellPanelError> {
    panel.place_above_system_menu_bar(bounds)
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    struct RecordingPanel {
        placed_bounds: Cell<Option<ShellPanelBounds>>,
    }

    impl ShellPanel for RecordingPanel {
        fn place_above_system_menu_bar(
            &self,
            bounds: ShellPanelBounds,
        ) -> Result<ShellPanelPlacement, ShellPanelError> {
            self.placed_bounds.set(Some(bounds));
            Ok(ShellPanelPlacement { top_offset: 0.0 })
        }
    }

    #[test]
    fn places_the_requested_shell_panel() {
        let panel = RecordingPanel {
            placed_bounds: Cell::new(None),
        };
        let bounds = ShellPanelBounds {
            left: 12.0,
            top: 0.0,
            width: 1488.0,
            height: 32.0,
        };

        let placement = place_shell_panel(&panel, bounds).expect("fake panel placement succeeds");

        assert_eq!(panel.placed_bounds.get(), Some(bounds));
        assert_eq!(placement, ShellPanelPlacement { top_offset: 0.0 });
    }
}
