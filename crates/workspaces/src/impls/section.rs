//! Composition root: wires concrete adapters into the section view. The only
//! module that may name both adapters and UI.

use std::sync::Arc;

use gpui::{App, AppContext, Entity};

use crate::adapters::aerospace::AerospaceProvider;
use crate::adapters::app_icons::AppKitIconProvider;
use crate::adapters::menus::AccessibilityMenuProvider;
use crate::app::bar::BarService;
use crate::ui::section::WorkspacesSection;

/// The workspaces/app-menu section backed by the AeroSpace window manager,
/// AppKit application icons, and the macOS Accessibility menu source.
pub fn aerospace_section(cx: &mut App) -> Entity<WorkspacesSection> {
    let service = Arc::new(BarService::new(
        Arc::new(AerospaceProvider::new()),
        Arc::new(AccessibilityMenuProvider::new()),
        Arc::new(AppKitIconProvider::new()),
    ));
    cx.new(|cx| WorkspacesSection::new(service, cx))
}
