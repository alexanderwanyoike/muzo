mod api;
mod credentials;

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use reqwest::Url;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub use api::{Client, Entry, Token};
pub use credentials::Credentials;

#[derive(Debug, thiserror::Error)]
pub enum DropboxError {
    #[error("Enter your Dropbox app key.")]
    MissingClient,
    #[error("Enter the authorization code from Dropbox.")]
    MissingCode,
    #[error("Dropbox returned invalid account information.")]
    InvalidAccount,
    #[error("Dropbox is unavailable or rejected the request.")]
    Transport(#[from] reqwest::Error),
    #[error("Dropbox's authorization address is invalid.")]
    Address,
    #[error("Could not access the OS credential store: {0}")]
    Credential(#[from] keyring::Error),
    #[error("Could not access the Dropbox audio cache: {0}")]
    Cache(#[from] std::io::Error),
    #[error("The Dropbox track exceeds the audio cache limit.")]
    CacheLimit,
    #[error("Dropbox authorization needs to be renewed for this account.")]
    Reconnect,
    #[error("Dropbox returned an invalid folder cursor.")]
    Cursor,
}

pub struct Authorization {
    client: String,
    verifier: String,
    state: Uuid,
}

impl Authorization {
    pub fn new(client: &str) -> Result<Self, DropboxError> {
        let client = client.trim();
        if client.is_empty() {
            return Err(DropboxError::MissingClient);
        }
        Ok(Self {
            client: client.into(),
            verifier: format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple()),
            state: Uuid::new_v4(),
        })
    }

    pub fn verifier(&self) -> &str {
        &self.verifier
    }
    pub fn client_id(&self) -> &str {
        &self.client
    }

    pub fn url(&self) -> Result<Url, DropboxError> {
        let mut url = Url::parse("https://www.dropbox.com/oauth2/authorize")
            .map_err(|_| DropboxError::Address)?;
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(self.verifier.as_bytes()));
        url.query_pairs_mut()
            .append_pair("client_id", &self.client)
            .append_pair("response_type", "code")
            .append_pair("token_access_type", "offline")
            .append_pair("scope", "files.metadata.read files.content.read")
            .append_pair("state", &self.state.to_string())
            .append_pair("code_challenge_method", "S256")
            .append_pair("code_challenge", &challenge);
        Ok(url)
    }
}
