use gpui::{
    App, Bounds, Context, Entity, Window, WindowBackgroundAppearance, WindowKind, WindowOptions,
    div, prelude::*, px, rgb, size,
};

use workspaces::ui::section::WorkspacesSection;

use battery::ui::section::BatterySection;

use crate::ui::island::IslandSection;

// Hidden standalone widgets are intentionally preserved for easy restoration:
// use bluetooth::ui::section::BluetoothSection;
// use notifications::ui::section::NotificationsSection;
// use theme::ui::section::ThemeSection;
// use volume::ui::section::VolumeSection;
// use wallpaper::ui::section::WallpaperSection;
// use wifi::ui::section::WifiSection;

pub const BAR_HEIGHT: f32 = 32.0;
const CORNER_RADIUS: f32 = 12.0;
const BAR_INSET: f32 = 12.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BarGeometry {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone)]
pub struct BarSections {
    pub workspaces: Entity<WorkspacesSection>,
    pub battery: Entity<BatterySection>,
    pub island: Entity<IslandSection>,
    // pub notifications: Entity<NotificationsSection>,
    // pub volume: Entity<VolumeSection>,
    // pub bluetooth: Entity<BluetoothSection>,
    // pub wifi: Entity<WifiSection>,
    // pub wallpaper: Entity<WallpaperSection>,
    // pub theme: Entity<ThemeSection>,
}

struct Bar {
    sections: BarSections,
}

impl Render for Bar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // Workspaces stay at the left. Battery and the complete status island
        // stay at the right; the other standalone widgets remain hidden.
        div().size_full().flex().flex_col().child(
            div()
                .h(px(BAR_HEIGHT))
                .flex_none()
                .flex()
                .items_center()
                .px_3()
                .bg(rgb(0x24283b))
                .rounded(px(CORNER_RADIUS))
                .text_color(rgb(0xc0caf5))
                .text_sm()
                .child(self.sections.workspaces.clone())
                .child(div().flex_1())
                // .child(self.sections.notifications.clone())
                // .child(self.sections.volume.clone())
                // .child(self.sections.bluetooth.clone())
                // .child(self.sections.wifi.clone())
                .child(self.sections.battery.clone())
                // .child(self.sections.wallpaper.clone())
                // .child(self.sections.theme.clone())
                .child(self.sections.island.clone()),
        )
    }
}

pub fn open(cx: &mut App, sections: BarSections) -> Result<BarGeometry, String> {
    let display_bounds = cx
        .primary_display()
        .map(|display| display.bounds())
        .ok_or_else(|| "no primary display is available".to_owned())?;

    let bar_width = f32::from(display_bounds.size.width) - BAR_INSET * 2.0;
    let panel_height = f32::from(display_bounds.size.height);
    let bar_bounds = Bounds {
        origin: display_bounds.origin,
        size: size(px(bar_width), px(panel_height)),
    };

    cx.open_window(
        WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(bar_bounds)),
            titlebar: None,
            focus: false,
            show: true,
            kind: WindowKind::PopUp,
            is_movable: false,
            is_resizable: false,
            is_minimizable: false,
            window_background: WindowBackgroundAppearance::Transparent,
            ..Default::default()
        },
        |_, cx| cx.new(|_| Bar { sections }),
    )
    .map_err(|error| format!("failed to open bar window: {error}"))?;

    Ok(BarGeometry {
        left: BAR_INSET as f64,
        top: 0.0,
        width: bar_width as f64,
        height: panel_height as f64,
    })
}
