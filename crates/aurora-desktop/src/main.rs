//! Aurora Dev Suite: the desktop application (Tauri shell and composition root).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod events;
mod logging;
mod state;

fn main() {
    let _log_guard = logging::init();
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "starting Aurora");

    let result = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state::AppState::new(state::AppState::default_config_dir()))
        .invoke_handler(tauri::generate_handler![
            commands::list_projects,
            commands::add_project,
            commands::pick_and_add_project,
            commands::rename_project,
            commands::remove_project,
            commands::relocate_project,
            commands::engines_overview,
            commands::refresh_engines,
            commands::get_settings,
            commands::save_settings,
            commands::project_cycle_state,
        ])
        .run(tauri::generate_context!());

    if let Err(error) = result {
        tracing::error!(%error, "Aurora could not start");
        eprintln!("Aurora could not start: {error}");
        if cfg!(target_os = "linux") {
            eprintln!("{}", logging::LINUX_WEBVIEW_HINT);
        }
        std::process::exit(1);
    }
}
