use std::fmt;

use crate::core::workspace::{WorkspaceId, WorkspaceObservation};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkspaceProviderError {
    /// The window manager is not installed, not running, or could not be
    /// spawned at all.
    Unavailable(String),
    /// The window manager ran but rejected the request.
    Failed(String),
    /// The window manager replied with output we could not understand.
    Malformed(String),
}

impl fmt::Display for WorkspaceProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(detail) => write!(f, "window manager unavailable: {detail}"),
            Self::Failed(detail) => write!(f, "window manager command failed: {detail}"),
            Self::Malformed(detail) => write!(f, "window manager output malformed: {detail}"),
        }
    }
}

/// The window-manager side of the bar: read workspace/window/focus state and
/// forward focus commands. Adapters own every manager-specific detail.
pub trait WorkspaceProvider: Send + Sync {
    /// One reading of workspaces, windows, and focus.
    fn observe(&self) -> Result<WorkspaceObservation, WorkspaceProviderError>;

    /// Ask the window manager to focus the given workspace.
    fn focus_workspace(&self, workspace: &WorkspaceId) -> Result<(), WorkspaceProviderError>;
}
