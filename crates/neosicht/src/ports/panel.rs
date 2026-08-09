#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PanelFrame {
    pub x: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

pub trait Panel {
    fn pin(&self, frame: PanelFrame) -> f64;
}
