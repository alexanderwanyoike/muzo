#[cfg(target_os = "linux")]
pub mod appimage;
mod update;
pub use update::{Artifact, Release, Update, UpdateError, latest, update_for};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Target {
    LinuxX64,
    WindowsX64,
    MacArm64,
    MacX64,
}

impl Target {
    pub(super) const ALL: [Self; 4] = [
        Self::LinuxX64,
        Self::WindowsX64,
        Self::MacArm64,
        Self::MacX64,
    ];

    pub(super) fn manifest_identity(self) -> (&'static str, &'static str) {
        match self {
            Self::LinuxX64 => ("linux", "x64"),
            Self::WindowsX64 => ("windows", "x64"),
            Self::MacArm64 => ("macos", "arm64"),
            Self::MacX64 => ("macos", "x64"),
        }
    }

    pub fn current() -> Option<Self> {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("linux", "x86_64") => Some(Self::LinuxX64),
            ("windows", "x86_64") => Some(Self::WindowsX64),
            ("macos", "aarch64") => Some(Self::MacArm64),
            ("macos", "x86_64") => Some(Self::MacX64),
            _ => None,
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::LinuxX64 => "AppImage",
            Self::WindowsX64 => "exe",
            Self::MacArm64 | Self::MacX64 => "dmg",
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::LinuxX64 => "linux-x64",
            Self::WindowsX64 => "windows-x64",
            Self::MacArm64 => "macos-arm64",
            Self::MacX64 => "macos-x64",
        }
    }
}

pub fn filename(version: &str, target: Target) -> String {
    let suffix = match target {
        Target::LinuxX64 => "linux-x64.AppImage",
        Target::WindowsX64 => "windows-x64-setup.exe",
        Target::MacArm64 => "macos-arm64.dmg",
        Target::MacX64 => "macos-x64.dmg",
    };
    format!("Muzo-{version}-{suffix}")
}
