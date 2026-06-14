//! Infrastructure layer.
//!
//! Concrete implementations of ports defined in `domain` and used by
//! `application`. Filesystem adapters, Dropbox adapter, persistence, etc.

pub mod sqlite_library_repository;
