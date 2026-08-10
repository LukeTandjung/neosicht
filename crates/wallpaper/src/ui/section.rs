//! The wallpaper widget: a picture-frame button opening the swatch-grid
//! popover. Placeholder-only for now — swatches approximate the design's
//! generated gradients until a wallpaper engine renders real thumbnails.

use base_gpui::popover::{
    PopoverAlign, PopoverHandle, PopoverPopup, PopoverPortal, PopoverPositioner, PopoverRoot,
    PopoverSide, PopoverTrigger, create_popover_handle,
};
use gpui::{
    Background, Context, EventEmitter, Window, div, linear_color_stop, linear_gradient, prelude::*,
    px, rgba, svg,
};
use theme::core::{palette, typography};

/// Vertical room, in pixels, the shell must clear below the bar row while the
/// wallpaper popover is open.
pub const POPUP_EXTENT: f64 = 260.0;

pub const WALLPAPER_ICON: &str = "wallpaper/wallpaper.svg";

pub const ASSETS: [(&str, &[u8]); 1] = [(WALLPAPER_ICON, include_bytes!("icons/wallpaper.svg"))];

const NAMES: [&str; 8] = [
    "RIDGE", "AURORA", "WEAVE", "DUSK", "BLOOM", "FOLD", "FLAT", "DUNE",
];

/// Linear-gradient approximations of the design's generated wallpapers.
fn swatch_fill(index: usize) -> Background {
    let (angle, from, to) = match index % 8 {
        0 => (160., palette::bg(), palette::raise()),
        1 => (200., palette::accent(), palette::bg()),
        2 => (115., palette::bar(), palette::bg()),
        3 => (180., palette::bar(), palette::bg()),
        4 => (0., palette::cyan(), palette::bg()),
        5 => (210., palette::bg(), palette::raise()),
        6 => (0., palette::bg(), palette::bg()),
        _ => (45., palette::bg(), palette::bar()),
    };
    let mut from = from;
    // The tinted wallpapers blend a faint accent hue over the base surface.
    if matches!(index % 8, 1 | 4) {
        from.a = 0.3;
    }
    linear_gradient(
        angle,
        linear_color_stop(from, 0.),
        linear_color_stop(to, 1.),
    )
}

pub enum SectionEvent {
    /// The section needs `extent` pixels of window below the bar row
    /// (0 = collapsed back to the bare bar).
    PopupExtentChanged { extent: f64 },
}

pub struct WallpaperSection {
    selected: usize,
    open: bool,
    popup_extent: f64,
    popover: PopoverHandle<()>,
}

impl EventEmitter<SectionEvent> for WallpaperSection {}

impl WallpaperSection {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            selected: 0,
            open: false,
            popup_extent: 0.0,
            popover: create_popover_handle(),
        }
    }

    fn set_popup_extent(&mut self, extent: f64, cx: &mut Context<Self>) {
        if self.popup_extent == extent {
            return;
        }
        self.popup_extent = extent;
        cx.emit(SectionEvent::PopupExtentChanged { extent });
    }

    fn render_swatch(&self, index: usize, cx: &mut Context<Self>) -> impl IntoElement {
        let selected = index == self.selected;
        div()
            .id(("wallpaper-swatch", index))
            .relative()
            .h(px(56.))
            .rounded(px(8.))
            .border_2()
            .border_color(if selected {
                palette::accent()
            } else {
                palette::transparent()
            })
            .overflow_hidden()
            .bg(swatch_fill(index))
            .cursor_pointer()
            .on_click(cx.listener(move |section, _event, _window, cx| {
                section.selected = index;
                cx.notify();
            }))
            .child(
                div()
                    .absolute()
                    .bottom_0()
                    .left_0()
                    .right_0()
                    .px(px(5.))
                    .py(px(3.))
                    .bg(linear_gradient(
                        180.,
                        linear_color_stop(rgba(0x0000009e), 0.),
                        linear_color_stop(rgba(0x00000000), 1.),
                    ))
                    .font_family(typography::mono())
                    .text_size(px(8.5))
                    .text_color(gpui::white())
                    .child(NAMES[index]),
            )
    }

    fn render_grid(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut cells: Vec<gpui::AnyElement> = (0..8)
            .map(|index| self.render_swatch(index, cx).into_any_element())
            .collect();
        cells.push(
            div()
                .h(px(56.))
                .rounded(px(8.))
                .border_1()
                .border_dashed()
                .border_color(palette::muted())
                .flex()
                .items_center()
                .justify_center()
                .text_center()
                .font_family(typography::mono())
                .text_size(px(8.5))
                .text_color(palette::muted())
                .child("DROP IMAGE")
                .into_any_element(),
        );

        let mut rows = Vec::new();
        let mut cells = cells.into_iter().peekable();
        while cells.peek().is_some() {
            rows.push(
                div().flex().gap(px(8.)).children(
                    cells
                        .by_ref()
                        .take(3)
                        .map(|cell| div().flex_1().child(cell)),
                ),
            );
        }

        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .mt(px(10.))
            .children(rows)
    }
}

impl Render for WallpaperSection {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let on_open_change = {
            let entity = cx.entity().downgrade();
            move |open: bool, _details: &mut _, _window: &mut Window, cx: &mut gpui::App| {
                entity
                    .update(cx, |section: &mut Self, cx| {
                        section.open = open;
                        section.set_popup_extent(if open { POPUP_EXTENT } else { 0.0 }, cx);
                        cx.notify();
                    })
                    .ok();
            }
        };

        let popover = PopoverRoot::<()>::new()
            .id("wallpaper")
            .handle(self.popover.clone())
            .on_open_change(on_open_change)
            .child(
                PopoverTrigger::new()
                    .id("wallpaper-trigger")
                    .aria_label("Wallpaper")
                    .flex()
                    .items_center()
                    .h(px(22.))
                    .px(px(8.))
                    .rounded(px(6.))
                    .style_with_state(|state, trigger| {
                        if state.open {
                            trigger.bg(palette::raise())
                        } else {
                            trigger
                                .bg(palette::transparent())
                                .hover(|style| style.bg(palette::raise()))
                        }
                    })
                    .child(
                        svg()
                            .path(WALLPAPER_ICON)
                            .size(px(15.))
                            .text_color(palette::text()),
                    ),
            )
            .child(
                PopoverPortal::new().child(
                    PopoverPositioner::new()
                        .side(PopoverSide::Bottom)
                        .align(PopoverAlign::End)
                        .side_offset(px(8.))
                        .collision_padding(px(0.))
                        .child(
                            PopoverPopup::new()
                                .id("wallpaper-popup")
                                .w(px(310.))
                                .p(px(12.))
                                .rounded(px(12.))
                                .bg(palette::bar())
                                .border_1()
                                .border_color(palette::border())
                                .shadow_lg()
                                .child_any(
                                    div()
                                        .font_family(typography::mono())
                                        .text_size(px(9.5))
                                        .text_color(palette::muted())
                                        .child("WALLPAPER")
                                        .into_any_element(),
                                )
                                .child_any(self.render_grid(cx).into_any_element()),
                        ),
                ),
            );

        div()
            .id("wallpaper-hover")
            .on_hover(cx.listener(|section, hovered: &bool, _window, cx| {
                if *hovered {
                    section.set_popup_extent(POPUP_EXTENT, cx);
                } else if !section.open {
                    section.set_popup_extent(0.0, cx);
                }
            }))
            .child(popover)
    }
}
