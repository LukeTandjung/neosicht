use gpui::{
    App, Bounds, Context, Entity, Window, WindowBackgroundAppearance, WindowKind, WindowOptions,
    div, prelude::*, px, rgb, size,
};
use workspaces::ui::section::{POPUP_EXTENT, WorkspacesSection};

pub const BAR_HEIGHT: f32 = 32.0;
const PANEL_HEIGHT: f32 = BAR_HEIGHT + POPUP_EXTENT as f32;
const CORNER_RADIUS: f32 = 12.0;
const BAR_INSET: f32 = 12.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BarGeometry {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

struct Bar {
    workspaces: Entity<WorkspacesSection>,
}

impl Render for Bar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // The window can be taller than the bar while a popover is open; only
        // the top row draws, the rest stays transparent.
        div().size_full().flex().flex_col().child(
            div()
                .h(px(BAR_HEIGHT))
                .flex_none()
                .flex()
                .items_center()
                .gap_3()
                .px_3()
                .bg(rgb(0x24283b))
                .rounded(px(CORNER_RADIUS))
                .text_color(rgb(0xc0caf5))
                .text_sm()
                .child(self.workspaces.clone()),
        )
    }
}

pub fn open(cx: &mut App, workspaces: Entity<WorkspacesSection>) -> Result<BarGeometry, String> {
    let display_bounds = cx
        .primary_display()
        .map(|display| display.bounds())
        .ok_or_else(|| "no primary display is available".to_owned())?;

    let bar_width = f32::from(display_bounds.size.width) - BAR_INSET * 2.0;
    let bar_bounds = Bounds {
        origin: display_bounds.origin,
        size: size(px(bar_width), px(PANEL_HEIGHT)),
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
        |_, cx| cx.new(|_| Bar { workspaces }),
    )
    .map_err(|error| format!("failed to open bar window: {error}"))?;

    Ok(BarGeometry {
        left: BAR_INSET as f64,
        top: 0.0,
        width: bar_width as f64,
        height: PANEL_HEIGHT as f64,
    })
}
