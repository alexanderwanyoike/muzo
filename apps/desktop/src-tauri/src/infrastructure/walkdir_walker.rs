use std::path::Path;

use walkdir::WalkDir;

use crate::application::scan_library::{FilesystemWalker, WalkError, WalkedFile};

/// Audio file extensions Muzo recognises. Anything else is ignored by the
/// walker. Kept as a flat list rather than a Set because the lookup happens
/// once per file via extension match; the list is small and stable.
pub const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "m4a", "ogg", "opus", "wav"];

/// `FilesystemWalker` backed by the `walkdir` crate. Walks the filesystem
/// recursively, filters by [`AUDIO_EXTENSIONS`], and returns each audio file
/// with its current size and mtime.
pub struct WalkdirWalker;

impl WalkdirWalker {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WalkdirWalker {
    fn default() -> Self {
        Self::new()
    }
}

impl FilesystemWalker for WalkdirWalker {
    fn walk_audio_files(&self, root: &Path) -> Result<Vec<WalkedFile>, WalkError> {
        if !root.exists() {
            return Err(WalkError::NotFound(root.display().to_string()));
        }

        let mut files = Vec::new();
        for entry in WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if !is_audio_file(path) {
                continue;
            }
            let metadata = entry.metadata().map_err(|e| WalkError::Io(e.to_string()))?;
            let mtime = metadata
                .modified()
                .map_err(|e| WalkError::Io(e.to_string()))?
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .map_err(|e| WalkError::Io(e.to_string()))?;
            let size = metadata.len();
            files.push(WalkedFile {
                path: path.to_path_buf(),
                size,
                mtime,
            });
        }
        Ok(files)
    }
}

fn is_audio_file(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => AUDIO_EXTENSIONS.contains(&ext.to_lowercase().as_str()),
        None => false,
    }
}
