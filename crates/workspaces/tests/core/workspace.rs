use workspaces::core::workspace::{
    HUE_SLOT_COUNT, WindowRecord, WorkspaceId, WorkspaceObservation, app_hue_slot, app_initial,
    assemble,
};

fn workspace_id(id: &str) -> WorkspaceId {
    WorkspaceId::new(id)
}

fn window(workspace: &str, app_name: &str) -> WindowRecord {
    WindowRecord {
        workspace: workspace_id(workspace),
        app_name: app_name.to_owned(),
        bundle_id: None,
    }
}

#[test]
fn assembles_windows_onto_listed_workspaces_in_order() {
    let snapshot = assemble(WorkspaceObservation {
        workspaces: vec![workspace_id("1"), workspace_id("2"), workspace_id("3")],
        windows: vec![
            window("3", "Ghostty"),
            window("1", "Firefox"),
            window("3", "Zed"),
        ],
        focused_workspace: Some(workspace_id("3")),
        frontmost_app: Some("Ghostty".to_owned()),
    });

    let ids: Vec<&str> = snapshot
        .workspaces
        .iter()
        .map(|workspace| workspace.id.as_str())
        .collect();
    assert_eq!(ids, ["1", "3"]);

    let apps_on = |index: usize| -> Vec<&str> {
        snapshot.workspaces[index]
            .tiles
            .iter()
            .map(|tile| tile.app_name.as_str())
            .collect()
    };
    assert_eq!(apps_on(0), ["Firefox"]);
    assert_eq!(apps_on(1), ["Ghostty", "Zed"]);

    assert_eq!(snapshot.focused, Some(workspace_id("3")));
    assert_eq!(snapshot.frontmost_app, Some("Ghostty".to_owned()));
}

#[test]
fn empty_unfocused_workspaces_are_hidden() {
    let snapshot = assemble(WorkspaceObservation {
        workspaces: vec![workspace_id("1"), workspace_id("2"), workspace_id("3")],
        windows: vec![window("2", "Mail")],
        focused_workspace: Some(workspace_id("2")),
        frontmost_app: None,
    });

    let ids: Vec<&str> = snapshot
        .workspaces
        .iter()
        .map(|workspace| workspace.id.as_str())
        .collect();
    assert_eq!(ids, ["2"]);
}

#[test]
fn the_focused_workspace_is_hidden_when_empty() {
    let snapshot = assemble(WorkspaceObservation {
        workspaces: vec![workspace_id("1"), workspace_id("2")],
        windows: vec![window("1", "Firefox")],
        focused_workspace: Some(workspace_id("2")),
        frontmost_app: None,
    });

    let ids: Vec<&str> = snapshot
        .workspaces
        .iter()
        .map(|workspace| workspace.id.as_str())
        .collect();
    assert_eq!(ids, ["1"]);
}

#[test]
fn each_window_earns_its_own_tile_even_for_the_same_app() {
    let snapshot = assemble(WorkspaceObservation {
        workspaces: vec![workspace_id("3")],
        windows: vec![window("3", "Ghostty"), window("3", "Ghostty")],
        ..WorkspaceObservation::default()
    });

    assert_eq!(snapshot.workspaces[0].tiles.len(), 2);
}

#[test]
fn window_on_unlisted_workspace_appends_that_workspace() {
    let snapshot = assemble(WorkspaceObservation {
        workspaces: vec![workspace_id("1")],
        windows: vec![window("1", "Zed"), window("9", "Mail"), window("9", "Chat")],
        ..WorkspaceObservation::default()
    });

    let ids: Vec<&str> = snapshot
        .workspaces
        .iter()
        .map(|workspace| workspace.id.as_str())
        .collect();
    assert_eq!(ids, ["1", "9"]);
    assert_eq!(snapshot.workspaces[1].tiles.len(), 2);
}

#[test]
fn empty_observation_is_an_empty_snapshot() {
    let snapshot = assemble(WorkspaceObservation::default());

    assert!(snapshot.workspaces.is_empty());
    assert_eq!(snapshot.focused, None);
    assert_eq!(snapshot.frontmost_app, None);
}

#[test]
fn icon_key_prefers_the_bundle_id() {
    let snapshot = assemble(WorkspaceObservation {
        workspaces: vec![workspace_id("1")],
        windows: vec![
            WindowRecord {
                workspace: workspace_id("1"),
                app_name: "kitty".to_owned(),
                bundle_id: Some("net.kovidgoyal.kitty".to_owned()),
            },
            window("1", "Unbundled"),
        ],
        ..WorkspaceObservation::default()
    });

    let tiles = &snapshot.workspaces[0].tiles;
    assert_eq!(tiles[0].icon_key, "net.kovidgoyal.kitty");
    assert_eq!(tiles[1].icon_key, "Unbundled");
}

#[test]
fn initial_is_the_first_alphanumeric_character_uppercased() {
    assert_eq!(app_initial("firefox"), "F");
    assert_eq!(app_initial("Zed"), "Z");
    assert_eq!(app_initial("1Password"), "1");
    assert_eq!(app_initial("— Notes"), "N");
}

#[test]
fn initial_falls_back_to_question_mark() {
    assert_eq!(app_initial(""), "?");
    assert_eq!(app_initial("···"), "?");
}

#[test]
fn hue_slot_is_stable_and_in_range() {
    assert_eq!(app_hue_slot("Ghostty"), app_hue_slot("Ghostty"));
    assert!(app_hue_slot("Ghostty") < HUE_SLOT_COUNT);
    assert!(app_hue_slot("") < HUE_SLOT_COUNT);
}
