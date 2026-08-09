use std::cell::Cell;
use std::rc::Rc;

use neosicht::{
    app::panel::ShellPanelService,
    ports::panel::{
        ShellPanel, ShellPanelBounds, ShellPanelError, ShellPanelInteraction, ShellPanelPlacement,
    },
};

#[derive(Clone)]
struct RecordingPanel {
    placed_bounds: Rc<Cell<Option<ShellPanelBounds>>>,
    interaction: Rc<Cell<Option<ShellPanelInteraction>>>,
}

impl ShellPanel for RecordingPanel {
    fn place_above_system_menu_bar(
        &self,
        bounds: ShellPanelBounds,
    ) -> Result<ShellPanelPlacement, ShellPanelError> {
        self.placed_bounds.set(Some(bounds));
        Ok(ShellPanelPlacement { top_offset: 0.0 })
    }

    fn set_interaction(&self, interaction: ShellPanelInteraction) -> Result<(), ShellPanelError> {
        self.interaction.set(Some(interaction));
        Ok(())
    }
}

fn recording_panel() -> RecordingPanel {
    RecordingPanel {
        placed_bounds: Rc::new(Cell::new(None)),
        interaction: Rc::new(Cell::new(None)),
    }
}

#[test]
fn initialization_places_the_panel_and_limits_input_to_the_bar() {
    let panel = recording_panel();
    let bounds = ShellPanelBounds {
        left: 12.0,
        top: 0.0,
        width: 1200.0,
        height: 372.0,
    };
    let service = ShellPanelService::new(panel.clone(), 32.0);

    let placement = service.initialize(bounds).unwrap();

    assert_eq!(panel.placed_bounds.get(), Some(bounds));
    assert_eq!(
        panel.interaction.get(),
        Some(ShellPanelInteraction::BarOnly { bar_height: 32.0 })
    );
    assert_eq!(placement, ShellPanelPlacement { top_offset: 0.0 });
}

#[test]
fn popup_visibility_controls_extended_input() {
    let panel = recording_panel();
    let service = ShellPanelService::new(panel.clone(), 32.0);

    service.set_popup_visible(true).unwrap();
    assert_eq!(
        panel.interaction.get(),
        Some(ShellPanelInteraction::Extended)
    );

    service.set_popup_visible(false).unwrap();
    assert_eq!(
        panel.interaction.get(),
        Some(ShellPanelInteraction::BarOnly { bar_height: 32.0 })
    );
}
