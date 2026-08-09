use crate::{adapters::panel::MacOsPanel, app::panel::PanelApp, ui};

pub fn run() {
    let panel_app = PanelApp::new(MacOsPanel);
    ui::bar::run(panel_app);
}
