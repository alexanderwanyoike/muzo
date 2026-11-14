use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

use reqwest::{Url, blocking::Client};
use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::{Target, filename};

const RELEASE_ROOT: &str = "https://github.com/alexanderwanyoike/muzo/releases";
const MAX_SAFE_RELEASE_SIZE: u64 = 9_007_199_254_740_991;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub schema_version: u32,
    pub tag: String,
    pub version: String,
    pub source_commit: String,
    pub artifacts: Vec<Artifact>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Artifact {
    pub name: String,
    pub platform: String,
    pub arch: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone)]
pub struct Update {
    pub version: Version,
    pub asset: Artifact,
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("The update manifest could not be read.")]
    InvalidManifest,
    #[error("The downloaded file did not match the release checksum.")]
    Integrity,
    #[error("Could not reach the update server.")]
    Network(#[from] reqwest::Error),
    #[error("Could not save the update: {0}")]
    Io(#[from] std::io::Error),
    #[error("The update was installed. Reopen {path} to finish. {source}")]
    Relaunch {
        path: PathBuf,
        source: std::io::Error,
    },
}

pub fn update_for(
    installed: &str,
    target: Target,
    release: Release,
) -> Result<Option<Update>, UpdateError> {
    let parse = |value| Version::parse(value).map_err(|_| UpdateError::InvalidManifest);
    let installed = parse(installed)?;
    let version = parse(&release.version)?;
    if release.schema_version != 1
        || release.tag != format!("v{version}")
        || !version.pre.is_empty()
        || !version.build.is_empty()
        || !valid_hex(&release.source_commit, 40)
    {
        return Err(UpdateError::InvalidManifest);
    }
    if release.artifacts.len() != Target::ALL.len() {
        return Err(UpdateError::InvalidManifest);
    }
    let mut selected = None;
    for required in Target::ALL {
        let (platform, arch) = required.manifest_identity();
        let expected = filename(&release.version, required);
        let Some(asset) = release.artifacts.iter().find(|asset| {
            asset.name == expected
                && asset.platform == platform
                && asset.arch == arch
                && asset.size > 0
                && asset.size <= MAX_SAFE_RELEASE_SIZE
                && valid_hex(&asset.sha256, 64)
        }) else {
            return Err(UpdateError::InvalidManifest);
        };
        if required == target {
            selected = Some(asset.clone());
        }
    }
    let asset = selected.ok_or(UpdateError::InvalidManifest)?;
    Ok((installed < version).then_some(Update { version, asset }))
}

impl Update {
    pub fn verify(&self, path: &Path) -> Result<(), UpdateError> {
        self.verify_and_copy(path, &mut std::io::sink())
    }

    pub fn verify_and_copy(&self, path: &Path, output: &mut impl Write) -> Result<(), UpdateError> {
        let mut digest = Sha256::new();
        let mut remaining = self.asset.size;
        let mut file = fs::File::open(path)?;
        let mut buffer = [0u8; 32 * 1024];
        loop {
            let size = file.read(&mut buffer)?;
            if size == 0 {
                break;
            }
            remaining = remaining
                .checked_sub(size as u64)
                .ok_or(UpdateError::Integrity)?;
            digest.update(&buffer[..size]);
            output.write_all(&buffer[..size])?;
        }
        if remaining != 0
            || !format!("{:x}", digest.finalize()).eq_ignore_ascii_case(&self.asset.sha256)
        {
            return Err(UpdateError::Integrity);
        }
        Ok(())
    }

    pub fn download_url(&self) -> Result<Url, UpdateError> {
        Url::parse(&format!(
            "{RELEASE_ROOT}/download/v{}/{}",
            self.version, self.asset.name
        ))
        .map_err(|_| UpdateError::InvalidManifest)
    }

    pub fn release_page(&self) -> String {
        format!("{RELEASE_ROOT}/tag/v{}", self.version)
    }

    pub fn download(
        &self,
        directory: &Path,
        progress: impl FnMut(u8),
    ) -> Result<PathBuf, UpdateError> {
        let body = client()?
            .get(self.download_url()?)
            .send()?
            .error_for_status()?;
        self.persist_download(directory, body, progress)
    }

    pub(crate) fn persist_download(
        &self,
        directory: &Path,
        mut body: impl Read,
        mut progress: impl FnMut(u8),
    ) -> Result<PathBuf, UpdateError> {
        fs::create_dir_all(directory)?;
        let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
        let mut digest = Sha256::new();
        let mut remaining = self.asset.size;
        let mut buffer = [0u8; 32 * 1024];
        let mut last_reported = 0;
        progress(0);
        loop {
            let read = body.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            remaining = remaining
                .checked_sub(read as u64)
                .ok_or(UpdateError::Integrity)?;
            digest.update(&buffer[..read]);
            temporary.write_all(&buffer[..read])?;
            if self.asset.size != 0 {
                let downloaded = u128::from(self.asset.size - remaining);
                let percent = (downloaded * 100 / u128::from(self.asset.size)).min(99) as u8;
                if percent != last_reported {
                    last_reported = percent;
                    progress(percent);
                }
            }
        }
        if remaining != 0
            || !format!("{:x}", digest.finalize()).eq_ignore_ascii_case(&self.asset.sha256)
        {
            return Err(UpdateError::Integrity);
        }
        temporary.as_file().sync_all()?;
        let path = directory.join(&self.asset.name);
        temporary.persist(&path).map_err(|error| error.error)?;
        progress(100);
        Ok(path)
    }
}

#[derive(Deserialize)]
struct Latest {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<PublishedAsset>,
}

#[derive(Deserialize)]
struct PublishedAsset {
    name: String,
    browser_download_url: String,
}

fn required_assets(version: &str) -> impl Iterator<Item = String> + '_ {
    Target::ALL
        .into_iter()
        .map(move |target| filename(version, target))
        .chain([
            format!("muzo-{version}-release.json"),
            "SHA256SUMS".into(),
            "latest.yml".into(),
            "latest-linux.yml".into(),
        ])
}

impl Latest {
    fn complete(&self, version: &str) -> bool {
        !self.draft
            && !self.prerelease
            && self.tag_name == format!("v{version}")
            && required_assets(version).all(|name| {
                let url = format!("{RELEASE_ROOT}/download/v{version}/{name}");
                self.assets
                    .iter()
                    .filter(|asset| asset.name == name && asset.browser_download_url == url)
                    .count()
                    == 1
            })
    }
}

pub fn latest(installed: &str, target: Target) -> Result<Option<Update>, UpdateError> {
    let client = client()?;
    let latest: Latest = client
        .get("https://api.github.com/repos/alexanderwanyoike/muzo/releases/latest")
        .send()?
        .error_for_status()?
        .json()?;
    let version = latest
        .tag_name
        .strip_prefix('v')
        .ok_or(UpdateError::InvalidManifest)?;
    let parsed = Version::parse(version).map_err(|_| UpdateError::InvalidManifest)?;
    let installed_version = Version::parse(installed).map_err(|_| UpdateError::InvalidManifest)?;
    if parsed <= installed_version {
        return Ok(None);
    }
    if !latest.complete(version) {
        return Err(UpdateError::InvalidManifest);
    }
    let manifest: Release = client
        .get(format!(
            "{RELEASE_ROOT}/download/v{parsed}/muzo-{parsed}-release.json"
        ))
        .send()?
        .error_for_status()?
        .json()?;
    if manifest.tag != latest.tag_name {
        return Err(UpdateError::InvalidManifest);
    }
    update_for(installed, target, manifest)
}

fn client() -> Result<Client, reqwest::Error> {
    Client::builder()
        .user_agent("Muzo Desktop")
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(180))
        .build()
}

fn valid_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod publication_tests {
    use super::*;

    #[test]
    fn it_rejects_an_update_when_the_published_assets_omit_an_installer() {
        let version = "0.1.0";
        let mut release = Latest {
            tag_name: format!("v{version}"),
            draft: false,
            prerelease: false,
            assets: Vec::new(),
        };
        for name in required_assets(version) {
            release.assets.push(PublishedAsset {
                browser_download_url: format!("{RELEASE_ROOT}/download/v{version}/{name}"),
                name,
            });
        }
        assert!(release.complete(version));
        release
            .assets
            .retain(|asset| asset.name != filename(version, Target::LinuxX64));
        assert!(!release.complete(version));
    }
}
