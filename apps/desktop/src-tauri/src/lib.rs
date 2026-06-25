//! Muzo desktop entrypoint.
//!
//! Layered per Clean Architecture (inner -> outer):
//!   domain -> application -> infrastructure -> commands
//!
//! See AGENTS.md for the full layering rules.

use std::sync::Arc;

use infrastructure::audio_stream_server::AudioStreamServer;
use infrastructure::lofty_metadata_reader::LoftyMetadataReader;
use infrastructure::migrations::MIGRATIONS;
use infrastructure::sqlite_library_repository::SqliteLibraryRepository;
use infrastructure::sqlite_track_repository::SqliteTrackRepository;
use infrastructure::walkdir_walker::WalkdirWalker;

pub mod application;
mod commands;
pub mod domain;
pub mod infrastructure;

/// Shared application state, injected into Tauri commands via `State<AppState>`.
pub struct AppState {
    pub library_repository: Arc<SqliteLibraryRepository>,
    pub track_repository: Arc<SqliteTrackRepository>,
    pub walker: Arc<WalkdirWalker>,
    pub metadata_reader: Arc<LoftyMetadataReader>,
    pub audio_stream_server: Arc<AudioStreamServer>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_data_dir = dirs::data_dir().expect("could not resolve app data dir");
    let muzo_dir = app_data_dir.join("muzo");
    std::fs::create_dir_all(&muzo_dir).expect("could not create muzo data dir");

    let db_path = muzo_dir.join("muzo.sqlite");
    let mut connection = rusqlite::Connection::open(&db_path)
        .unwrap_or_else(|e| panic!("could not open muzo sqlite at {:?}: {}", db_path, e));
    MIGRATIONS
        .to_latest(&mut connection)
        .expect("could not migrate sqlite schema");
    let audio_stream_server =
        AudioStreamServer::start().expect("could not start audio stream server");

    let state = AppState {
        library_repository: Arc::new(SqliteLibraryRepository::new(connection)),
        track_repository: Arc::new(SqliteTrackRepository::new(
            rusqlite::Connection::open(&db_path).expect("could not open tracks sqlite connection"),
        )),
        walker: Arc::new(WalkdirWalker::new()),
        metadata_reader: Arc::new(LoftyMetadataReader::new()),
        audio_stream_server: Arc::new(audio_stream_server),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::ping::ping,
            commands::add_library::add_library,
            commands::list_libraries::list_libraries,
            commands::scan_library::scan_library,
            commands::scan_library::list_tracks,
            commands::prepare_track_audio_source::prepare_track_audio_source,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
