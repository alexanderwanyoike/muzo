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
        || !valid_hex(&release.source_commit, 40)
    {
        return Err(UpdateError::InvalidManifest);
    }
    if installed >= version {
        return Ok(None);
    }

    let expected = filename(&release.version, target);
    let mut matching = release
        .artifacts
        .into_iter()
        .filter(|asset| asset.name == expected);
    let asset = matching.next().ok_or(UpdateError::InvalidManifest)?;
    if matching.next().is_some() || asset.size == 0 || !valid_hex(&asset.sha256, 64) {
        return Err(UpdateError::InvalidManifest);
    }
    Ok(Some(Update { version, asset }))
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

    pub fn download(&self, directory: &Path) -> Result<PathBuf, UpdateError> {
        fs::create_dir_all(directory)?;
        let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
        let mut body = client()?
            .get(self.download_url()?)
            .send()?
            .error_for_status()?;
        let mut digest = Sha256::new();
        let mut remaining = self.asset.size;
        let mut buffer = [0u8; 32 * 1024];
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
        }
        if remaining != 0
            || !format!("{:x}", digest.finalize()).eq_ignore_ascii_case(&self.asset.sha256)
        {
            return Err(UpdateError::Integrity);
        }
        temporary.as_file().sync_all()?;
        let path = directory.join(&self.asset.name);
        temporary.persist(&path).map_err(|error| error.error)?;
        Ok(path)
    }
}

#[derive(Deserialize)]
struct Latest {
    tag_name: String,
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
    value.len() == length && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
