//! AeroSpace window-manager adapter. Every AeroSpace-specific detail — the
//! binary name, CLI flags, and output format — lives here, behind the
//! `WorkspaceProvider` port. Parsing is kept in pure functions so it can be
//! tested against captured CLI output without a running window manager.

use std::process::Command;

use crate::core::workspace::{WindowRecord, WorkspaceId, WorkspaceObservation};
use crate::ports::workspaces::{WorkspaceProvider, WorkspaceProviderError};

/// Separates the fields we ask aerospace to print. A literal tab cannot
/// realistically appear in a workspace name, bundle id, or application name.
const FIELD_SEPARATOR: char = '\t';
/// `%{workspace-is-focused}` folds the focused-workspace query into the
/// listing so one observation costs three aerospace calls, not four.
const WORKSPACE_FORMAT: &str = "%{workspace}\t%{workspace-is-focused}";
const WINDOW_FORMAT: &str = "%{workspace}\t%{app-bundle-id}\t%{app-name}";

pub struct AerospaceProvider {
    binary: String,
}

impl AerospaceProvider {
    pub fn new() -> Self {
        Self::with_binary(option_env!("NEOSICHT_AEROSPACE_BIN").unwrap_or("aerospace"))
    }

    /// Override the binary path, e.g. when aerospace is not on PATH.
    pub fn with_binary(binary: impl Into<String>) -> Self {
        Self {
            binary: binary.into(),
        }
    }

    fn run(&self, args: &[&str]) -> Result<String, WorkspaceProviderError> {
        let output = Command::new(&self.binary)
            .args(args)
            .output()
            .map_err(|error| {
                WorkspaceProviderError::Unavailable(format!(
                    "failed to run {}: {error}",
                    self.binary
                ))
            })?;

        if !output.status.success() {
            return Err(WorkspaceProviderError::Failed(format!(
                "{} {} exited with {}: {}",
                self.binary,
                args.join(" "),
                output.status,
                String::from_utf8_lossy(&output.stderr).trim(),
            )));
        }

        String::from_utf8(output.stdout).map_err(|_| {
            WorkspaceProviderError::Malformed("aerospace produced non-UTF-8 output".to_owned())
        })
    }

    fn frontmost_app(&self) -> Result<Option<String>, WorkspaceProviderError> {
        // `--focused` fails when the focused workspace has no windows; that is
        // an empty chip, not an error.
        match self.run(&["list-windows", "--focused", "--format", "%{app-name}"]) {
            Ok(output) => {
                let name = output.trim();
                Ok((!name.is_empty()).then(|| name.to_owned()))
            }
            Err(WorkspaceProviderError::Failed(_)) => Ok(None),
            Err(other) => Err(other),
        }
    }
}

impl Default for AerospaceProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceProvider for AerospaceProvider {
    fn observe(&self) -> Result<WorkspaceObservation, WorkspaceProviderError> {
        let (workspaces, focused_workspace) = parse_workspaces(&self.run(&[
            "list-workspaces",
            "--all",
            "--format",
            WORKSPACE_FORMAT,
        ])?)?;
        let windows =
            parse_windows(&self.run(&["list-windows", "--all", "--format", WINDOW_FORMAT])?)?;
        let frontmost_app = self.frontmost_app()?;

        Ok(WorkspaceObservation {
            workspaces,
            windows,
            focused_workspace,
            frontmost_app,
        })
    }

    fn focus_workspace(&self, workspace: &WorkspaceId) -> Result<(), WorkspaceProviderError> {
        self.run(&["workspace", workspace.as_str()]).map(|_| ())
    }
}

/// Parse `list-workspaces` output in `WORKSPACE_FORMAT`: one
/// `<workspace>\t<true|false>` pair per line. Returns the workspaces in
/// listing order plus whichever one is flagged focused.
pub fn parse_workspaces(
    output: &str,
) -> Result<(Vec<WorkspaceId>, Option<WorkspaceId>), WorkspaceProviderError> {
    let mut workspaces = Vec::new();
    let mut focused = None;
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((workspace, is_focused)) = line.split_once(FIELD_SEPARATOR) else {
            return Err(WorkspaceProviderError::Malformed(format!(
                "expected '<workspace>\\t<is-focused>' in list-workspaces output, got {line:?}"
            )));
        };
        let id = WorkspaceId::new(workspace.trim());
        if is_focused.trim() == "true" {
            focused = Some(id.clone());
        }
        workspaces.push(id);
    }
    Ok((workspaces, focused))
}

/// Parse `list-windows` output in `WINDOW_FORMAT`: one
/// `<workspace>\t<bundle-id>\t<app-name>` triple per line. The bundle id may
/// be empty; everything after the second tab is the application name.
pub fn parse_windows(output: &str) -> Result<Vec<WindowRecord>, WorkspaceProviderError> {
    let mut records = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut fields = line.splitn(3, FIELD_SEPARATOR);
        let (Some(workspace), Some(bundle_id), Some(app_name)) =
            (fields.next(), fields.next(), fields.next())
        else {
            return Err(WorkspaceProviderError::Malformed(format!(
                "expected '<workspace>\\t<bundle-id>\\t<app-name>' in list-windows output, got {line:?}"
            )));
        };
        let bundle_id = bundle_id.trim();
        records.push(WindowRecord {
            workspace: WorkspaceId::new(workspace.trim()),
            app_name: app_name.trim().to_owned(),
            bundle_id: (!bundle_id.is_empty()).then(|| bundle_id.to_owned()),
        });
    }
    Ok(records)
}
