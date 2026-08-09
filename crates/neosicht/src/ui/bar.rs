//! Experiment 0/2 — non-activating shell panel, placed at the very top.
//!
//! Exp 0 (PASS) proved `WindowKind::PopUp` never steals focus. This iteration
//! adds the Barik "replace" placement: a full-screen transparent panel with an
//! opaque bar drawn at the top, pushed to a low window level (below apps, so it
//! doesn't intercept their clicks) via the native/pin.m C shim. The full-screen
//! frame is what lets it reach y=0 despite AppKit's menu-bar clamp.
//!
//! Verify placement with CGWindowList (expect the panel at y=0, full height).

use gpui::{
    App, Bounds, Context, Window, WindowBackgroundAppearance, WindowKind, WindowOptions, div,
    prelude::*, px, rgb, size,
};
use gpui_platform::application;

use crate::{
    app::panel::PanelApp,
    ports::panel::{Panel, PanelFrame},
};

// Full-width bar flush at the top (covers the native menu-bar band), with all
// four corners rounded to match the screen bezel radius. The window itself is
// transparent, so the corner cutouts show through.
const BAR_HEIGHT: f32 = 32.0;
const CORNER_RADIUS: f32 = 12.0;
// Inset each side so the bar is narrower than the screen; a full-width bar's
// left/right corners hide under the physical bezel, so the radius never shows.
const BAR_INSET: f32 = 12.0;

struct Bar {
    clicks: usize,
}

impl Render for Bar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_3()
            .size_full()
            .px_3()
            .bg(rgb(0x24283b))
            .rounded(px(CORNER_RADIUS))
            .text_color(rgb(0xc0caf5))
            .text_sm()
            .child("neosicht exp-panel — top strip")
            .child(
                div()
                    .id("click-test")
                    .px_2()
                    .rounded_md()
                    .bg(rgb(0x2f3549))
                    .hover(|style| style.bg(rgb(0x444b6a)))
                    .cursor_pointer()
                    .child(format!("clicks: {}", self.clicks))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.clicks += 1;
                        cx.notify();
                    })),
            )
    }
}

pub fn run<P>(panel_app: PanelApp<P>)
where
    P: Panel + 'static,
{
    application().run(move |cx: &mut App| {
        let display_bounds = cx
            .primary_display()
            .map(|display| display.bounds())
            .expect("no primary display");

        // Bar inset from both edges so its rounded corners clear the bezel.
        let bar_width = f32::from(display_bounds.size.width) - BAR_INSET * 2.0;
        let bar = Bounds {
            origin: display_bounds.origin,
            size: size(px(bar_width), px(BAR_HEIGHT)),
        };

        cx.open_window(
            WindowOptions {
                window_bounds: Some(gpui::WindowBounds::Windowed(bar)),
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
            |_, cx| cx.new(|_| Bar { clicks: 0 }),
        )
        .expect("failed to open bar window");

        // Raise above the menu bar and pin flush to the top edge.
        let top_offset = panel_app.pin(PanelFrame {
            x: BAR_INSET as f64,
            top: 0.0,
            width: bar_width as f64,
            height: BAR_HEIGHT as f64,
        });
        println!("pinned bar; top-edge offset from screen top = {top_offset} (0 = flush)");

        // Deliberately no cx.activate(true): the shell must never activate itself.
    });
}
