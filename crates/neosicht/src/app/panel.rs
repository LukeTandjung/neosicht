use crate::ports::panel::{Panel, PanelFrame};

pub struct PanelApp<P> {
    panel: P,
}

impl<P> PanelApp<P>
where
    P: Panel,
{
    pub fn new(panel: P) -> Self {
        Self { panel }
    }

    pub fn pin(&self, frame: PanelFrame) -> f64 {
        self.panel.pin(frame)
    }
}
