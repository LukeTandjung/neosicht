use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::core::menu::AppMenu;
use crate::core::workspace::{self, Snapshot, WorkspaceId};
use crate::ports::icons::AppIconProvider;
use crate::ports::menus::{MenuProvider, MenuProviderError};
use crate::ports::workspaces::{WorkspaceProvider, WorkspaceProviderError};

const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Everything the bar section renders in one pass.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BarModel {
    pub snapshot: Snapshot,
    /// Menus of the frontmost application. Empty when no application is
    /// frontmost or the menu source is degraded — the menu source must never
    /// block the workspace pills.
    pub menus: Vec<AppMenu>,
    /// PNG icon bytes per `AppTile::icon_key`. A tile whose key is absent
    /// falls back to its initial-letter rendering.
    pub icons: HashMap<String, Arc<Vec<u8>>>,
}

/// Application boundary for bar observation and user actions. It owns the
/// polling policy and all external capabilities; presentation code only asks
/// for models and sends intents.
pub struct BarService {
    workspaces: Arc<dyn WorkspaceProvider>,
    menus: Arc<dyn MenuProvider>,
    icons: Arc<dyn AppIconProvider>,
}

impl BarService {
    pub fn new(
        workspaces: Arc<dyn WorkspaceProvider>,
        menus: Arc<dyn MenuProvider>,
        icons: Arc<dyn AppIconProvider>,
    ) -> Self {
        Self {
            workspaces,
            menus,
            icons,
        }
    }

    pub fn load(&self) -> Result<BarModel, WorkspaceProviderError> {
        let snapshot = workspace::assemble(self.workspaces.observe()?);
        let menu_list = snapshot
            .frontmost_app
            .as_deref()
            .map(|app_name| self.menus.menus_for(app_name).unwrap_or_default())
            .unwrap_or_default();

        let mut icon_map = HashMap::new();
        for workspace in &snapshot.workspaces {
            for tile in &workspace.tiles {
                if icon_map.contains_key(&tile.icon_key) {
                    continue;
                }
                if let Some(png) = self
                    .icons
                    .icon_png(tile.bundle_id.as_deref(), &tile.app_name)
                {
                    icon_map.insert(tile.icon_key.clone(), png);
                }
            }
        }

        Ok(BarModel {
            snapshot,
            menus: menu_list,
            icons: icon_map,
        })
    }

    /// Wait for the application-owned poll interval, then take one observation.
    /// Callers run this blocking operation on their background executor.
    pub fn poll(&self) -> Result<BarModel, WorkspaceProviderError> {
        thread::sleep(POLL_INTERVAL);
        self.load()
    }

    pub fn focus(&self, workspace: &WorkspaceId) -> Result<BarModel, WorkspaceProviderError> {
        self.workspaces.focus_workspace(workspace)?;
        self.load()
    }

    pub fn activate(
        &self,
        app_name: &str,
        menu_title: &str,
        item_index: usize,
    ) -> Result<(), MenuProviderError> {
        self.menus.activate(app_name, menu_title, item_index)
    }
}
