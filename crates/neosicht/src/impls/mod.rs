use gpui::App;
use gpui_platform::application;

use crate::{
    adapters::panel::AppKitShellPanel, app::panel::place_shell_panel,
    ports::panel::ShellPanelBounds, ui,
};

pub fn run() {
    application().run(|cx: &mut App| {
        let geometry = match ui::bar::open(cx) {
            Ok(geometry) => geometry,
            Err(error) => {
                eprintln!("{error}");
                cx.quit();
                return;
            }
        };

        let bounds = ShellPanelBounds {
            left: geometry.left,
            top: geometry.top,
            width: geometry.width,
            height: geometry.height,
        };

        match place_shell_panel(&AppKitShellPanel, bounds) {
            Ok(placement) => println!(
                "placed shell panel; top-edge offset = {}",
                placement.top_offset
            ),
            Err(error) => {
                eprintln!("failed to place shell panel: {error:?}");
                cx.quit();
            }
        }

        // Activating neosicht would steal keyboard focus from the foreground app.
    });
}
