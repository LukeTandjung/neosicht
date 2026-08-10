use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{App, AppContext};
use gpui_platform::application;

use crate::{
    adapters::panel::AppKitShellPanel, app::panel::ShellPanelService,
    ports::panel::ShellPanelBounds, ui,
};

pub fn run() {
    application()
        .with_assets(ui::assets::ShellAssets)
        .run(|cx: &mut App| {
            base_gpui::init(cx);

            let sections = ui::bar::BarSections {
                workspaces: workspaces::impls::section::aerospace_section(cx),
                battery: battery::impls::section::iokit_section(cx),
                island: cx.new(ui::island::IslandSection::new),
                // notifications: cx.new(notifications::ui::section::NotificationsSection::new),
                // volume: cx.new(volume::ui::section::VolumeSection::new),
                // bluetooth: cx.new(bluetooth::ui::section::BluetoothSection::new),
                // wifi: cx.new(wifi::ui::section::WifiSection::new),
                // wallpaper: cx.new(wallpaper::ui::section::WallpaperSection::new),
                // theme: cx.new(theme::ui::section::ThemeSection::new),
            };

            let geometry = match ui::bar::open(cx, sections.clone()) {
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

            let extents = Rc::new(RefCell::new([0.0_f64; 2]));

            macro_rules! track_extent {
                ($entity:expr, $event:path, $slot:expr) => {
                    let extents = extents.clone();
                    let panel = panel.clone();
                    cx.subscribe(&$entity, move |_section, event, _cx| {
                        let $event { extent } = event;
                        extents.borrow_mut()[$slot] = *extent;
                        let visible = extents.borrow().iter().any(|&extent| extent > 0.0);
                        if let Err(error) = panel.set_popup_visible(visible) {
                            eprintln!("failed to update shell panel input: {error:?}");
                        }
                    })
                    .detach();
                };
            }

            track_extent!(
                sections.workspaces,
                workspaces::ui::section::SectionEvent::PopupExtentChanged,
                0
            );
            track_extent!(
                sections.island,
                ui::island::IslandEvent::PopupExtentChanged,
                1
            );
            // When restoring hidden widgets, expand `extents` and restore their
            // corresponding `track_extent!` subscriptions here as well.

            // Activating neosicht would steal keyboard focus from the foreground app.
        });
}
