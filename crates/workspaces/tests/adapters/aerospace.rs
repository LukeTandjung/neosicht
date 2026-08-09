use workspaces::adapters::aerospace::{parse_windows, parse_workspaces};
use workspaces::core::workspace::{WindowRecord, WorkspaceId};
use workspaces::ports::workspaces::WorkspaceProviderError;

#[test]
fn parses_workspace_names_and_the_focused_flag() {
    let (ids, focused) = parse_workspaces("1\tfalse\n2\ttrue\n10\tfalse\n").unwrap();

    assert_eq!(
        ids,
        vec![
            WorkspaceId::new("1"),
            WorkspaceId::new("2"),
            WorkspaceId::new("10"),
        ]
    );
    assert_eq!(focused, Some(WorkspaceId::new("2")));
}

#[test]
fn no_focused_flag_means_no_focused_workspace() {
    let (ids, focused) = parse_workspaces("1\tfalse\n2\tfalse\n").unwrap();

    assert_eq!(ids.len(), 2);
    assert_eq!(focused, None);
}

#[test]
fn workspace_names_are_trimmed_and_blank_lines_skipped() {
    let (ids, _) = parse_workspaces(" mail \tfalse\n\n  2\ttrue\n").unwrap();

    assert_eq!(ids, vec![WorkspaceId::new("mail"), WorkspaceId::new("2")]);
}

#[test]
fn empty_output_is_no_workspaces() {
    let (ids, focused) = parse_workspaces("").unwrap();

    assert!(ids.is_empty());
    assert_eq!(focused, None);
}

#[test]
fn workspace_line_without_separator_is_malformed() {
    let error = parse_workspaces("nonsense\n").unwrap_err();

    assert!(matches!(error, WorkspaceProviderError::Malformed(_)));
}

#[test]
fn parses_workspace_bundle_and_app_triples() {
    let windows =
        parse_windows("3\tcom.mitchellh.ghostty\tGhostty\n1\torg.mozilla.firefox\tFirefox\n")
            .unwrap();

    assert_eq!(
        windows,
        vec![
            WindowRecord {
                workspace: WorkspaceId::new("3"),
                app_name: "Ghostty".to_owned(),
                bundle_id: Some("com.mitchellh.ghostty".to_owned()),
            },
            WindowRecord {
                workspace: WorkspaceId::new("1"),
                app_name: "Firefox".to_owned(),
                bundle_id: Some("org.mozilla.firefox".to_owned()),
            },
        ]
    );
}

#[test]
fn empty_bundle_id_becomes_none() {
    let windows = parse_windows("2\t\tSomeTool\n").unwrap();

    assert_eq!(windows[0].bundle_id, None);
    assert_eq!(windows[0].app_name, "SomeTool");
}

#[test]
fn app_names_keep_internal_whitespace() {
    let windows = parse_windows("2\tcom.microsoft.VSCode\tVisual Studio Code\n").unwrap();

    assert_eq!(windows[0].app_name, "Visual Studio Code");
}

#[test]
fn everything_after_the_second_tab_belongs_to_the_app_name() {
    let windows = parse_windows("2\tid\tWeird\tName\n").unwrap();

    assert_eq!(windows[0].app_name, "Weird\tName");
}

#[test]
fn blank_window_lines_are_skipped() {
    let windows = parse_windows("\n3\tid\tZed\n\n").unwrap();

    assert_eq!(windows.len(), 1);
}

#[test]
fn window_line_without_enough_fields_is_malformed() {
    let error = parse_windows("3\tonly-two-fields\n").unwrap_err();

    assert!(matches!(error, WorkspaceProviderError::Malformed(_)));
}
