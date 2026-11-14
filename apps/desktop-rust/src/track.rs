use std::path::{Path, PathBuf};

pub struct Track {
    path: PathBuf,
    title: String,
}

impl Track {
    pub fn read(path: PathBuf) -> Self {
        let title = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        Self { path, title }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn title(&self) -> &str {
        &self.title
    }
}
