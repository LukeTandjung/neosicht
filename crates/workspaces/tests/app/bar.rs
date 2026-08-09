use std::sync::{Arc, Mutex};

use workspaces::app::bar::BarService;
use workspaces::core::menu::{AppMenu, MenuEntry, MenuItemSpec};
use workspaces::core::workspace::{WindowRecord, WorkspaceId, WorkspaceObservation};
use workspaces::ports::icons::AppIconProvider;
use workspaces::ports::menus::{MenuProvider, MenuProviderError};
use workspaces::ports::workspaces::{WorkspaceProvider, WorkspaceProviderError};

struct InMemoryWorkspaces {
    observation: Result<WorkspaceObservation, WorkspaceProviderError>,
    focus_calls: Mutex<Vec<WorkspaceId>>,
}

impl InMemoryWorkspaces {
    fn returning(observation: Result<WorkspaceObservation, WorkspaceProviderError>) -> Self {
        Self {
            observation,
            focus_calls: Mutex::new(Vec::new()),
        }
    }
}

impl WorkspaceProvider for InMemoryWorkspaces {
    fn observe(&self) -> Result<WorkspaceObservation, WorkspaceProviderError> {
        self.observation.clone()
    }

    fn focus_workspace(&self, workspace: &WorkspaceId) -> Result<(), WorkspaceProviderError> {
        self.focus_calls.lock().unwrap().push(workspace.clone());
        Ok(())
    }
}

struct InMemoryMenus {
    menus: Result<Vec<AppMenu>, MenuProviderError>,
    activations: Mutex<Vec<(String, String, usize)>>,
}

impl InMemoryMenus {
    fn returning(menus: Result<Vec<AppMenu>, MenuProviderError>) -> Self {
        Self {
            menus,
            activations: Mutex::new(Vec::new()),
        }
    }
}

impl MenuProvider for InMemoryMenus {
    fn menus_for(&self, _app_name: &str) -> Result<Vec<AppMenu>, MenuProviderError> {
        self.menus.clone()
    }

    fn activate(
        &self,
        app_name: &str,
        menu_title: &str,
        item_index: usize,
    ) -> Result<(), MenuProviderError> {
        self.activations.lock().unwrap().push((
            app_name.to_owned(),
            menu_title.to_owned(),
            item_index,
        ));
        Ok(())
    }
}

/// Serves a one-byte PNG stand-in for the apps it knows, `None` otherwise.
struct InMemoryIcons {
    known: Vec<String>,
}

impl AppIconProvider for InMemoryIcons {
    fn icon_png(&self, bundle_id: Option<&str>, app_name: &str) -> Option<Arc<Vec<u8>>> {
        let key = bundle_id.unwrap_or(app_name);
        self.known
            .iter()
            .any(|known| known == key)
            .then(|| Arc::new(vec![0x89]))
    }
}

fn no_icons() -> InMemoryIcons {
    InMemoryIcons { known: Vec::new() }
}

fn service(
    workspaces: Arc<InMemoryWorkspaces>,
    menus: Arc<InMemoryMenus>,
    icons: InMemoryIcons,
) -> BarService {
    BarService::new(workspaces, menus, Arc::new(icons))
}

fn observation_with_frontmost(frontmost_app: Option<&str>) -> WorkspaceObservation {
    WorkspaceObservation {
        workspaces: vec![WorkspaceId::new("1"), WorkspaceId::new("2")],
        windows: vec![
            WindowRecord {
                workspace: WorkspaceId::new("1"),
                app_name: "Zed".to_owned(),
                bundle_id: Some("dev.zed.Zed".to_owned()),
            },
            WindowRecord {
                workspace: WorkspaceId::new("2"),
                app_name: "Mail".to_owned(),
                bundle_id: None,
            },
        ],
        focused_workspace: Some(WorkspaceId::new("1")),
        frontmost_app: frontmost_app.map(str::to_owned),
    }
}

fn file_menu() -> AppMenu {
    AppMenu {
        title: "File".to_owned(),
        entries: vec![
            MenuEntry::Item(MenuItemSpec::new("New File").shortcut("⌘N")),
            MenuEntry::Separator,
            MenuEntry::Item(MenuItemSpec::new("Save").shortcut("⌘S")),
        ],
    }
}

#[test]
fn loads_snapshot_and_menus_for_the_frontmost_app() {
    let workspaces = Arc::new(InMemoryWorkspaces::returning(Ok(
        observation_with_frontmost(Some("Zed")),
    )));
    let menus = Arc::new(InMemoryMenus::returning(Ok(vec![file_menu()])));

    let model = service(workspaces.clone(), menus.clone(), no_icons())
        .load()
        .unwrap();

    assert_eq!(model.snapshot.workspaces.len(), 2);
    assert_eq!(model.snapshot.frontmost_app, Some("Zed".to_owned()));
    assert_eq!(model.menus, vec![file_menu()]);
}

#[test]
fn resolved_icons_are_keyed_by_tile_icon_key() {
    let workspaces = Arc::new(InMemoryWorkspaces::returning(Ok(
        observation_with_frontmost(None),
    )));
    let menus = Arc::new(InMemoryMenus::returning(Ok(Vec::new())));
    let icons = InMemoryIcons {
        known: vec!["dev.zed.Zed".to_owned()],
    };

    let model = service(workspaces.clone(), menus.clone(), icons)
        .load()
        .unwrap();

    assert!(model.icons.contains_key("dev.zed.Zed"));
    // Mail has no bundle id and is unknown to the icon source: no entry, so
    // the tile falls back to its initial.
    assert!(!model.icons.contains_key("Mail"));
}

#[test]
fn no_frontmost_app_means_no_menus() {
    let workspaces = Arc::new(InMemoryWorkspaces::returning(Ok(
        observation_with_frontmost(None),
    )));
    let menus = Arc::new(InMemoryMenus::returning(Ok(vec![file_menu()])));

    let model = service(workspaces.clone(), menus.clone(), no_icons())
        .load()
        .unwrap();

    assert!(model.menus.is_empty());
}

#[test]
fn menu_source_failure_degrades_to_empty_menus() {
    let workspaces = Arc::new(InMemoryWorkspaces::returning(Ok(
        observation_with_frontmost(Some("Zed")),
    )));
    let menus = Arc::new(InMemoryMenus::returning(Err(
        MenuProviderError::Unavailable("no accessibility permission".to_owned()),
    )));

    let model = service(workspaces.clone(), menus.clone(), no_icons())
        .load()
        .unwrap();

    assert_eq!(model.snapshot.frontmost_app, Some("Zed".to_owned()));
    assert!(model.menus.is_empty());
}

#[test]
fn workspace_provider_failure_propagates() {
    let workspaces = Arc::new(InMemoryWorkspaces::returning(Err(
        WorkspaceProviderError::Unavailable("aerospace is not running".to_owned()),
    )));
    let menus = Arc::new(InMemoryMenus::returning(Ok(Vec::new())));

    let error = service(workspaces.clone(), menus.clone(), no_icons())
        .load()
        .unwrap_err();

    assert_eq!(
        error,
        WorkspaceProviderError::Unavailable("aerospace is not running".to_owned())
    );
}

#[test]
fn focus_workspace_forwards_to_the_provider() {
    let workspaces = Arc::new(InMemoryWorkspaces::returning(Ok(
        WorkspaceObservation::default(),
    )));

    service(
        workspaces.clone(),
        Arc::new(InMemoryMenus::returning(Ok(Vec::new()))),
        no_icons(),
    )
    .focus(&WorkspaceId::new("4"))
    .unwrap();

    assert_eq!(
        *workspaces.focus_calls.lock().unwrap(),
        vec![WorkspaceId::new("4")]
    );
}

#[test]
fn activate_menu_item_forwards_to_the_provider() {
    let menus = Arc::new(InMemoryMenus::returning(Ok(Vec::new())));

    service(
        Arc::new(InMemoryWorkspaces::returning(Ok(
            WorkspaceObservation::default(),
        ))),
        menus.clone(),
        no_icons(),
    )
    .activate("Zed", "File", 2)
    .unwrap();

    assert_eq!(
        *menus.activations.lock().unwrap(),
        vec![("Zed".to_owned(), "File".to_owned(), 2)]
    );
}
