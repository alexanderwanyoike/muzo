use std::{collections::HashSet, time::Duration};

use reqwest::{
    Method,
    blocking::{Client as Http, RequestBuilder, Response},
    header::RANGE,
};
use serde::Deserialize;
use serde_json::json;

use super::{Authorization, DropboxError};

const API: &str = "https://api.dropboxapi.com";
const CONTENT: &str = "https://content.dropboxapi.com";

pub struct Client {
    http: Http,
    api: String,
    content: String,
}

#[derive(Deserialize)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub account_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Entry {
    #[serde(rename = ".tag")]
    pub kind: String,
    pub name: String,
    pub path_display: Option<String>,
    pub path_lower: Option<String>,
    pub size: Option<i64>,
    pub server_modified: Option<String>,
    pub rev: Option<String>,
}

impl Entry {
    pub fn path(&self) -> Option<&str> {
        self.path_display.as_deref().or(self.path_lower.as_deref())
    }
}

#[derive(Deserialize)]
struct Page {
    entries: Vec<Entry>,
    cursor: String,
    has_more: bool,
}

impl Client {
    #[cfg(test)]
    pub(crate) fn for_endpoint(endpoint: &str) -> Result<Self, DropboxError> {
        Self::with_endpoints(endpoint, endpoint)
    }

    pub fn new() -> Result<Self, DropboxError> {
        Self::with_endpoints(API, CONTENT)
    }

    pub(super) fn with_endpoints(api: &str, content: &str) -> Result<Self, DropboxError> {
        Self::with_timeout(api, content, Duration::from_secs(120))
    }

    pub(crate) fn for_artwork(timeout: Duration) -> Result<Self, DropboxError> {
        Self::with_timeout(API, CONTENT, timeout)
    }

    fn with_timeout(api: &str, content: &str, timeout: Duration) -> Result<Self, DropboxError> {
        let http = Http::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(10))
            .timeout(timeout)
            .build()?;
        Ok(Self {
            http,
            api: api.into(),
            content: content.into(),
        })
    }

    pub fn exchange(
        &self,
        authorization: &Authorization,
        code: &str,
    ) -> Result<Token, DropboxError> {
        let code = code.trim();
        if code.is_empty() {
            return Err(DropboxError::MissingCode);
        }
        self.token(&[
            ("grant_type", "authorization_code"),
            ("client_id", authorization.client_id()),
            ("code", code),
            ("code_verifier", authorization.verifier()),
        ])
    }

    pub fn refresh(&self, client_id: &str, refresh_token: &str) -> Result<Token, DropboxError> {
        self.token(&[
            ("grant_type", "refresh_token"),
            ("client_id", client_id),
            ("refresh_token", refresh_token),
        ])
        .map_err(|error| match error {
            DropboxError::Transport(ref transport)
                if transport
                    .status()
                    .is_some_and(|status| status == 400 || status == 401) =>
            {
                DropboxError::Reconnect
            }
            other => other,
        })
    }

    fn token(&self, form: &[(&str, &str)]) -> Result<Token, DropboxError> {
        Ok(self
            .http
            .post(format!("{}/oauth2/token", self.api))
            .form(form)
            .send()?
            .error_for_status()?
            .json()?)
    }

    pub fn list(
        &self,
        token: &str,
        path: &str,
        recursive: bool,
    ) -> Result<Vec<Entry>, DropboxError> {
        let path = if path == "/" { "" } else { path };
        let mut page = self.rpc(
            token,
            "files/list_folder",
            json!({ "path": path, "recursive": recursive, "include_deleted": false }),
        )?;
        let mut entries = Vec::new();
        let mut seen = HashSet::new();
        loop {
            let Page {
                entries: batch,
                cursor,
                has_more,
            } = page;
            entries.extend(batch);
            if !has_more {
                return Ok(entries);
            }
            if cursor.is_empty() || !seen.insert(cursor.clone()) {
                return Err(DropboxError::Cursor);
            }
            page = self.rpc(
                token,
                "files/list_folder/continue",
                json!({ "cursor": cursor }),
            )?;
        }
    }

    fn rpc(
        &self,
        token: &str,
        route: &str,
        value: serde_json::Value,
    ) -> Result<Page, DropboxError> {
        Ok(self
            .http
            .post(format!("{}/2/{route}", self.api))
            .bearer_auth(token)
            .json(&value)
            .send()?
            .error_for_status()?
            .json()?)
    }

    pub fn download(&self, token: &str, path: &str) -> Result<Response, DropboxError> {
        Ok(self
            .download_request(Method::POST, token, path)
            .send()?
            .error_for_status()?)
    }

    pub(super) fn download_range(
        &self,
        token: &str,
        path: &str,
        start: u64,
        end: u64,
        timeout: Duration,
    ) -> Result<Response, reqwest::Error> {
        self.download_request(Method::GET, token, path)
            .header(RANGE, format!("bytes={start}-{end}"))
            .timeout(timeout)
            .send()?
            .error_for_status()
    }

    fn download_request(&self, method: Method, token: &str, path: &str) -> RequestBuilder {
        let json = json!({ "path": path }).to_string();
        let header: String = json
            .chars()
            .map(|character| {
                if character.is_ascii() {
                    character.to_string()
                } else {
                    character
                        .encode_utf16(&mut [0; 2])
                        .iter()
                        .map(|unit| format!("\\u{unit:04x}"))
                        .collect()
                }
            })
            .collect();
        self.http
            .request(method, format!("{}/2/files/download", self.content))
            .bearer_auth(token)
            .header("Dropbox-API-Arg", header)
    }
}
