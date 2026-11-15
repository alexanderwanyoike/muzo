use std::{
    fs,
    io::{self, Write},
    path::Path,
};

use super::{AppearanceError, legacy};

const DEFAULT: f32 = 240.0;
const MINIMUM_CONTENT: f32 = 520.0;
const STORAGE_KEY: &[u8] = b"muzo.sidebar-width.v1";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SidebarWidth(f32);

impl Default for SidebarWidth {
    fn default() -> Self {
        Self(DEFAULT)
    }
}

impl SidebarWidth {
    pub(crate) const MINIMUM: Self = Self(192.0);
    pub(crate) const MAXIMUM: Self = Self(360.0);

    pub fn new(width: f32) -> Self {
        if width.is_finite() {
            Self(width.round().clamp(Self::MINIMUM.0, Self::MAXIMUM.0))
        } else {
            Self::default()
        }
    }

    pub fn visible(self, viewport: f32) -> f32 {
        self.0
            .min((viewport - MINIMUM_CONTENT).clamp(Self::MINIMUM.0, Self::MAXIMUM.0))
    }
}

pub fn load_or_import(
    path: &Path,
    legacy_path: Option<&Path>,
) -> Result<SidebarWidth, AppearanceError> {
    let read = match fs::read(path) {
        Ok(data) => return decode(&data),
        Err(error) if error.kind() == io::ErrorKind::NotFound => legacy_path,
        Err(error) => return Err(error.into()),
    };
    if let Some(legacy_path) = read
        && let Some(value) = legacy::read_setting(legacy_path, STORAGE_KEY)?
    {
        let width = decode(value.as_bytes())?;
        save(path, width)?;
        return Ok(width);
    }
    Ok(SidebarWidth::default())
}

fn decode(data: &[u8]) -> Result<SidebarWidth, AppearanceError> {
    let width = serde_json::from_slice::<f32>(data)?;
    if !width.is_finite() || !(SidebarWidth::MINIMUM.0..=SidebarWidth::MAXIMUM.0).contains(&width) {
        return Err(AppearanceError::Invalid);
    }
    Ok(SidebarWidth::new(width))
}

pub fn save(path: &Path, width: SidebarWidth) -> Result<(), AppearanceError> {
    let directory = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(directory)?;
    let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
    temporary.write_all(&serde_json::to_vec(&width.0)?)?;
    temporary.as_file().sync_all()?;
    temporary.persist(path).map_err(|error| error.error)?;
    Ok(())
}
