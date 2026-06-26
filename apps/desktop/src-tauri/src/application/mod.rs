//! Application layer.
//!
//! Use cases (commands and queries) that orchestrate domain objects with
//! ports defined in the domain layer. Has no knowledge of Tauri, serde on
//! the wire, or any concrete adapter.

pub mod add_library;
pub mod dropbox_oauth;
pub mod edit_track_metadata;
pub mod error;
pub mod list_libraries;
pub mod ping;
pub mod playlists;
pub mod reconcile_filesystem_libraries;
pub mod resolve_track_audio_source;
pub mod scan_dropbox_library;
pub mod scan_library;
pub mod watch_filesystem_libraries;
