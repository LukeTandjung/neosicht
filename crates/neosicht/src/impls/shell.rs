use std::sync::Arc;

use gpui::App;
use gpui_platform::application;
use workspaces::ui::section::SectionEvent;

use crate::{
    adapters::panel::AppKitShellPanel, app::panel::ShellPanelService,
    ports::panel::ShellPanelBounds, ui,
};

pub fn run() {
    application().run(|cx: &mut App| {
        base_gpui::init(cx);

        let workspaces = workspaces::impls::section::aerospace_section(cx);

        let geometry = match ui::bar::open(cx, workspaces.clone()) {
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

        let panel = Arc::new(ShellPanelService::new(
            AppKitShellPanel,
            ui::bar::BAR_HEIGHT as f64,
        ));
        match panel.initialize(bounds) {
            Ok(placement) => println!(
                "placed shell panel; top-edge offset = {}",
                placement.top_offset
            ),
            Err(error) => {
                eprintln!("failed to place shell panel: {error:?}");
                cx.quit();
                return;
            }
        }

        // The panel stays tall enough for popovers. Outside that interaction,
        // the native adapter passes the transparent area through to apps.
        cx.subscribe(&workspaces, move |_section, event, _cx| {
            let SectionEvent::PopupExtentChanged { extent } = event;
            if let Err(error) = panel.set_popup_visible(*extent > 0.0) {
                eprintln!("failed to update shell panel input: {error:?}");
            }
        })
        .detach();

        // Activating neosicht would steal keyboard focus from the foreground app.
    });
}
