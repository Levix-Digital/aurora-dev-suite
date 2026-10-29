/// Aurora's commands; each one gets an `allow-<command>` permission that
/// `capabilities/default.json` grants to the main window.
const COMMANDS: &[&str] = &[
    "list_projects",
    "add_project",
    "pick_and_add_project",
    "rename_project",
    "remove_project",
    "relocate_project",
    "engines_overview",
    "refresh_engines",
    "get_settings",
    "save_settings",
];

fn main() {
    let attributes = tauri_build::Attributes::new()
        .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS));
    if let Err(error) = tauri_build::try_build(attributes) {
        panic!("tauri build failed: {error:#}");
    }
}
