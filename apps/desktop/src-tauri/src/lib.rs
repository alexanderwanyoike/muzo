//! Muzo desktop entrypoint.
//!
//! Layered per Clean Architecture (inner -> outer):
//!   domain -> application -> infrastructure -> commands
//!
//! See AGENTS.md for the full layering rules.

use std::sync::Arc;

use infrastructure::sqlite_library_repository::SqliteLibraryRepository;

mod application;
mod commands;
pub mod domain;
pub mod infrastructure;

/// Shared application state, injected into Tauri commands via `State<AppState>`.
pub struct AppState {
    pub library_repository: Arc<SqliteLibraryRepository>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_data_dir = dirs::data_dir().expect("could not resolve app data dir");
    let muzo_dir = app_data_dir.join("muzo");
    std::fs::create_dir_all(&muzo_dir).expect("could not create muzo data dir");

    let db_path = muzo_dir.join("muzo.sqlite");
    let connection = rusqlite::Connection::open(&db_path)
        .unwrap_or_else(|e| panic!("could not open muzo sqlite at {:?}: {}", db_path, e));
    SqliteLibraryRepository::migrate(&connection).expect("could not migrate muzo sqlite");

    let state = AppState {
        library_repository: Arc::new(SqliteLibraryRepository::new(connection)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::ping::ping,
            commands::add_library::add_library
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
