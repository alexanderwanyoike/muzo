//! Infrastructure layer.
//!
//! Concrete implementations of ports defined in `domain` and used by
//! `application`. Filesystem adapters, Dropbox adapter, persistence, etc.

pub mod audio_stream_server;
pub mod lofty_metadata_reader;
pub mod migrations;
pub mod notify_filesystem_watcher;
pub mod sqlite_library_repository;
pub mod sqlite_track_repository;
pub mod walkdir_walker;
