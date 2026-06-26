//! Muzo desktop entrypoint.
//!
//! Layered per Clean Architecture (inner -> outer):
//!   domain -> application -> infrastructure -> commands
//!
//! See AGENTS.md for the full layering rules.

use std::sync::Arc;
use std::time::Duration;

use application::reconcile_filesystem_libraries::reconcile_filesystem_libraries;
use application::scan_library::scan_library;
use application::watch_filesystem_libraries::{
    watch_filesystem_libraries, FilesystemLibraryChangeHandler,
};
use infrastructure::audio_stream_server::AudioStreamServer;
use infrastructure::lofty_metadata_reader::LoftyMetadataReader;
use infrastructure::migrations::MIGRATIONS;
use infrastructure::notify_filesystem_watcher::NotifyFilesystemLibraryWatcher;
use infrastructure::sqlite_library_repository::SqliteLibraryRepository;
use infrastructure::sqlite_play_history_repository::SqlitePlayHistoryRepository;
use infrastructure::sqlite_playlist_repository::SqlitePlaylistRepository;
use infrastructure::sqlite_track_repository::SqliteTrackRepository;
use infrastructure::walkdir_walker::WalkdirWalker;

pub mod application;
mod commands;
pub mod domain;
pub mod infrastructure;

/// Shared application state, injected into Tauri commands via `State<AppState>`.
pub struct AppState {
    pub library_repository: Arc<SqliteLibraryRepository>,
    pub play_history_repository: Arc<SqlitePlayHistoryRepository>,
    pub playlist_repository: Arc<SqlitePlaylistRepository>,
    pub track_repository: Arc<SqliteTrackRepository>,
    pub walker: Arc<WalkdirWalker>,
    pub metadata_reader: Arc<LoftyMetadataReader>,
    pub filesystem_watcher: Arc<NotifyFilesystemLibraryWatcher>,
    pub audio_stream_server: Arc<AudioStreamServer>,
}

pub(crate) fn filesystem_library_change_handler(
    library_repository: Arc<SqliteLibraryRepository>,
    track_repository: Arc<SqliteTrackRepository>,
    walker: Arc<WalkdirWalker>,
    metadata_reader: Arc<LoftyMetadataReader>,
) -> FilesystemLibraryChangeHandler {
    Arc::new(move |library_id| {
        if let Err(error) = scan_library(
            &library_id,
            &*library_repository,
            &*track_repository,
            &*walker,
            &*metadata_reader,
        ) {
            eprintln!(
                "filesystem library reconciliation failed for {}: {}",
                library_id, error
            );
        }
    })
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

    let library_repository = Arc::new(SqliteLibraryRepository::new(connection));
    let play_history_repository = Arc::new(SqlitePlayHistoryRepository::new(
        rusqlite::Connection::open(&db_path)
            .expect("could not open play history sqlite connection"),
    ));
    let playlist_repository = Arc::new(SqlitePlaylistRepository::new(
        rusqlite::Connection::open(&db_path).expect("could not open playlists sqlite connection"),
    ));
    let track_repository = Arc::new(SqliteTrackRepository::new(
        rusqlite::Connection::open(&db_path).expect("could not open tracks sqlite connection"),
    ));
    let walker = Arc::new(WalkdirWalker::new());
    let metadata_reader = Arc::new(LoftyMetadataReader::new());
    let filesystem_watcher = Arc::new(NotifyFilesystemLibraryWatcher::new(Duration::from_millis(
        500,
    )));

    match reconcile_filesystem_libraries(
        &*library_repository,
        &*track_repository,
        &*walker,
        &*metadata_reader,
    ) {
        Ok(report) => {
            for failure in report.failures {
                eprintln!(
                    "filesystem library reconciliation failed for {}: {}",
                    failure.library_id, failure.message
                );
            }
        }
        Err(error) => {
            eprintln!("filesystem library reconciliation failed: {}", error);
        }
    }

    let on_filesystem_library_change = filesystem_library_change_handler(
        Arc::clone(&library_repository),
        Arc::clone(&track_repository),
        Arc::clone(&walker),
        Arc::clone(&metadata_reader),
    );
    match watch_filesystem_libraries(
        &*library_repository,
        &*filesystem_watcher,
        on_filesystem_library_change,
    ) {
        Ok(report) => {
            for failure in report.failures {
                eprintln!(
                    "filesystem library watcher failed for {}: {}",
                    failure.library_id, failure.message
                );
            }
        }
        Err(error) => {
            eprintln!("filesystem library watcher setup failed: {}", error);
        }
    }

    let audio_stream_server =
        AudioStreamServer::start().expect("could not start audio stream server");

    let state = AppState {
        library_repository,
        play_history_repository,
        playlist_repository,
        track_repository,
        walker,
        metadata_reader,
        filesystem_watcher,
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
            commands::track_metadata::edit_track_metadata,
            commands::track_metadata::clear_track_metadata_override,
            commands::play_history::record_track_play,
            commands::play_history::list_track_play_counts,
            commands::playlists::create_playlist,
            commands::playlists::list_playlists,
            commands::playlists::add_track_to_playlist,
            commands::playlists::remove_playlist_entry,
            commands::playlists::reorder_playlist_entries,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
