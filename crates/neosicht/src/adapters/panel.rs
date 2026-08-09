use crate::ports::panel::{
    ShellPanel, ShellPanelBounds, ShellPanelError, ShellPanelInteraction, ShellPanelPlacement,
};

const CG_STATUS_WINDOW_LEVEL_KEY: i32 = 9;

#[derive(Clone, Copy, Default)]
pub struct AppKitShellPanel;

impl ShellPanel for AppKitShellPanel {
    fn place_above_system_menu_bar(
        &self,
        bounds: ShellPanelBounds,
    ) -> Result<ShellPanelPlacement, ShellPanelError> {
        #[link(name = "neosicht_native")]
        unsafe extern "C" {
            fn neosicht_pin_shell_window(
                cg_level_key: i32,
                x: f64,
                top: f64,
                width: f64,
                height: f64,
            ) -> f64;
        }

        let top_offset = unsafe {
            neosicht_pin_shell_window(
                CG_STATUS_WINDOW_LEVEL_KEY,
                bounds.left,
                bounds.top,
                bounds.width,
                bounds.height,
            )
        };

        if top_offset < 0.0 {
            Err(ShellPanelError::WindowUnavailable)
        } else {
            Ok(ShellPanelPlacement { top_offset })
        }
    }

    fn set_interaction(&self, interaction: ShellPanelInteraction) -> Result<(), ShellPanelError> {
        #[link(name = "neosicht_native")]
        unsafe extern "C" {
            fn neosicht_set_panel_interaction(extended: i8, bar_height: f64) -> i8;
        }

        let (extended, bar_height) = match interaction {
            ShellPanelInteraction::BarOnly { bar_height } => (false, bar_height),
            ShellPanelInteraction::Extended => (true, 0.0),
        };
        let updated = unsafe { neosicht_set_panel_interaction(i8::from(extended), bar_height) };
        if updated == 0 {
            Err(ShellPanelError::WindowUnavailable)
        } else {
            Ok(())
        }
    }
}
