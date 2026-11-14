mod legacy;
mod palette;
mod surfaces;
pub(crate) use surfaces::Surfaces;
pub mod sidebar;

use std::{
    fs,
    io::{self, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};

pub const MATERIALS: [(&str, &str); 8] = [
    ("studio", "Studio"),
    ("pulse", "Pulse"),
    ("aluminium", "Brushed aluminium"),
    ("paper", "Warm paper"),
    ("carbon", "Carbon fibre"),
    ("walnut", "Walnut"),
    ("porcelain", "Porcelain"),
    ("glass", "Frosted glass"),
];
pub const ACCENTS: [(&str, &str); 16] = [
    ("cobalt", "Cobalt"),
    ("copper", "Copper"),
    ("sage", "Sage"),
    ("plum", "Plum"),
    ("amber", "Amber"),
    ("teal", "Teal"),
    ("rose", "Rose"),
    ("slate", "Slate"),
    ("violet", "Violet"),
    ("ice", "Ice"),
    ("glow-green", "Glow green"),
    ("glow-cyan", "Glow cyan"),
    ("glow-blue", "Glow blue"),
    ("glow-violet", "Glow violet"),
    ("glow-pink", "Glow pink"),
    ("glow-orange", "Glow orange"),
];
pub const MODES: [(&str, &str); 2] = [("light", "Light"), ("dark", "Dark")];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Appearance {
    pub theme: String,
    pub mode: String,
    pub accent: String,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            theme: "studio".into(),
            mode: "light".into(),
            accent: "cobalt".into(),
        }
    }
}

impl Appearance {
    pub fn theme(&self) -> iced::Theme {
        palette::palette(&self.theme, self.mode == "dark", &self.accent)
    }

    pub(crate) fn surfaces(&self) -> Surfaces {
        surfaces::for_material(&self.theme, self.mode == "dark")
    }

    fn valid(&self) -> bool {
        MATERIALS.iter().any(|(id, _)| *id == self.theme)
            && MODES.iter().any(|(id, _)| *id == self.mode)
            && ACCENTS.iter().any(|(id, _)| *id == self.accent)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AppearanceError {
    #[error("Could not access appearance preferences: {0}")]
    Io(#[from] io::Error),
    #[error("Could not read appearance preferences: {0}")]
    Json(#[from] serde_json::Error),
    #[error("The saved appearance contains an unknown setting.")]
    Invalid,
    #[error("Could not read Electron appearance preferences: {0}")]
    Legacy(String),
}

pub fn load_or_import(
    target: &Path,
    electron_leveldb: &Path,
) -> Result<Appearance, AppearanceError> {
    if target.try_exists()? {
        return load(target);
    }
    if let Some(appearance) = legacy::read(electron_leveldb)? {
        save(target, &appearance)?;
        return Ok(appearance);
    }
    Ok(Appearance::default())
}

pub fn load(path: &Path) -> Result<Appearance, AppearanceError> {
    let data = match fs::read(path) {
        Ok(data) => data,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Appearance::default()),
        Err(error) => return Err(error.into()),
    };
    let appearance: Appearance = serde_json::from_slice(&data)?;
    if !appearance.valid() {
        return Err(AppearanceError::Invalid);
    }
    Ok(appearance)
}

pub fn save(path: &Path, appearance: &Appearance) -> Result<(), AppearanceError> {
    if !appearance.valid() {
        return Err(AppearanceError::Invalid);
    }
    let directory = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(directory)?;
    let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
    temporary.write_all(&serde_json::to_vec(appearance)?)?;
    temporary.as_file().sync_all()?;
    temporary.persist(path).map_err(|error| error.error)?;
    Ok(())
}
