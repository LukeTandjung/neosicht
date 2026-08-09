use std::fmt;

/// Number of distinct accent hues available to color application tiles and
/// workspace numbers. Matches the eight base16 accent slots of the design.
pub const HUE_SLOT_COUNT: usize = 8;

/// A window manager's name for a workspace. AeroSpace allows arbitrary
/// non-empty strings, not just digits, so this stays textual.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WorkspaceId(String);

impl WorkspaceId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WorkspaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// One window as the window manager reports it: which workspace it sits on
/// and which application owns it, in the provider's stacking order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowRecord {
    pub workspace: WorkspaceId,
    pub app_name: String,
    /// The owning application's bundle identifier, when the window manager
    /// knows it. Preferred over the name for icon lookup.
    pub bundle_id: Option<String>,
}

/// Everything a window-manager provider can tell us in one reading.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorkspaceObservation {
    pub workspaces: Vec<WorkspaceId>,
    pub windows: Vec<WindowRecord>,
    pub focused_workspace: Option<WorkspaceId>,
    pub frontmost_app: Option<String>,
}

/// One application tile inside a workspace pill — one per window, so the same
/// application appears once per window it has on that workspace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppTile {
    pub app_name: String,
    pub bundle_id: Option<String>,
    /// Stable key for icon lookup and caching: the bundle id when known,
    /// otherwise the application name.
    pub icon_key: String,
    pub initial: String,
    pub hue_slot: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Workspace {
    pub id: WorkspaceId,
    pub tiles: Vec<AppTile>,
}

/// The assembled workspace state the bar renders from.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Snapshot {
    pub workspaces: Vec<Workspace>,
    pub focused: Option<WorkspaceId>,
    pub frontmost_app: Option<String>,
}

/// The single character shown on an application tile: the first alphanumeric
/// character of the name, uppercased, or "?" when there is none.
pub fn app_initial(app_name: &str) -> String {
    app_name
        .chars()
        .find(|character| character.is_alphanumeric())
        .map(|character| character.to_uppercase().collect())
        .unwrap_or_else(|| "?".to_owned())
}

/// A stable accent slot for an application, so the same app is tinted the same
/// hue on every workspace and across refreshes. FNV-1a over the name.
pub fn app_hue_slot(app_name: &str) -> usize {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in app_name.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    (hash % HUE_SLOT_COUNT as u64) as usize
}

/// Assemble the bar's workspace model from one raw observation.
///
/// Workspaces keep the provider's order and windows keep their stacking order
/// within each workspace. A window on a workspace the provider failed to list
/// still earns that workspace a pill, appended in first-seen order, so the bar
/// never silently drops windows. Empty workspaces are hidden — window managers
/// list every configured workspace, and placeholders with no open windows add
/// noise even when that workspace is currently focused.
pub fn assemble(observation: WorkspaceObservation) -> Snapshot {
    let mut workspaces: Vec<Workspace> = observation
        .workspaces
        .into_iter()
        .map(|id| Workspace {
            id,
            tiles: Vec::new(),
        })
        .collect();

    for window in observation.windows {
        let index = match workspaces
            .iter()
            .position(|workspace| workspace.id == window.workspace)
        {
            Some(index) => index,
            None => {
                workspaces.push(Workspace {
                    id: window.workspace.clone(),
                    tiles: Vec::new(),
                });
                workspaces.len() - 1
            }
        };
        let icon_key = window
            .bundle_id
            .clone()
            .unwrap_or_else(|| window.app_name.clone());
        workspaces[index].tiles.push(AppTile {
            initial: app_initial(&window.app_name),
            hue_slot: app_hue_slot(&window.app_name),
            bundle_id: window.bundle_id,
            icon_key,
            app_name: window.app_name,
        });
    }

    let focused = observation.focused_workspace;
    workspaces.retain(|workspace| !workspace.tiles.is_empty());

    Snapshot {
        workspaces,
        focused,
        frontmost_app: observation.frontmost_app,
    }
}
