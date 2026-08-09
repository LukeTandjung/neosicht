use crate::ports::panel::{Panel, PanelFrame};

const CG_STATUS_WINDOW_LEVEL_KEY: i32 = 9;

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

#[derive(Clone, Copy, Default)]
pub struct MacOsPanel;

impl Panel for MacOsPanel {
    fn pin(&self, frame: PanelFrame) -> f64 {
        unsafe {
            neosicht_pin_shell_window(
                CG_STATUS_WINDOW_LEVEL_KEY,
                frame.x,
                frame.top,
                frame.width,
                frame.height,
            )
        }
    }
}
