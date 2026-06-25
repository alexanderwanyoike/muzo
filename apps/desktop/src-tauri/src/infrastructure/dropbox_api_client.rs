use serde::Deserialize;
use url::form_urlencoded;

const DROPBOX_TOKEN_URL: &str = "https://api.dropboxapi.com/oauth2/token";
const DROPBOX_LIST_FOLDER_URL: &str = "https://api.dropboxapi.com/2/files/list_folder";
const DROPBOX_LIST_FOLDER_CONTINUE_URL: &str =
    "https://api.dropboxapi.com/2/files/list_folder/continue";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropboxClientConfig {
    pub client_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropboxHttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropboxHttpResponse {
    pub status: u16,
    pub body: String,
}

pub trait DropboxHttpTransport {
    fn send(&self, request: DropboxHttpRequest) -> Result<DropboxHttpResponse, DropboxClientError>;
}

pub struct ReqwestDropboxHttpTransport {
    client: reqwest::blocking::Client,
}

impl ReqwestDropboxHttpTransport {
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
        }
    }
}

impl Default for ReqwestDropboxHttpTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl DropboxHttpTransport for ReqwestDropboxHttpTransport {
    fn send(&self, request: DropboxHttpRequest) -> Result<DropboxHttpResponse, DropboxClientError> {
        let method = request
            .method
            .parse::<reqwest::Method>()
            .map_err(|error| DropboxClientError::Http(error.to_string()))?;
        let mut builder = self.client.request(method, request.url);
        for (name, value) in request.headers {
            builder = builder.header(name, value);
        }
        let response = builder
            .body(request.body)
            .send()
            .map_err(|error| DropboxClientError::Http(error.to_string()))?;
        let status = response.status().as_u16();
        let body = response
            .text()
            .map_err(|error| DropboxClientError::Http(error.to_string()))?;
        Ok(DropboxHttpResponse { status, body })
    }
}

pub struct DropboxApiClient<'a, T>
where
    T: DropboxHttpTransport + ?Sized,
{
    config: DropboxClientConfig,
    transport: &'a T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropboxToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub account_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropboxEntry {
    pub tag: DropboxEntryTag,
    pub name: String,
    pub id: String,
    pub path_lower: Option<String>,
    pub path_display: Option<String>,
    pub server_modified: Option<String>,
    pub rev: Option<String>,
    pub size: Option<u64>,
    pub content_hash: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropboxEntryTag {
    File,
    Folder,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DropboxClientError {
    Http(String),
    UnexpectedStatus(u16, String),
    InvalidResponse(String),
}

impl<'a, T> DropboxApiClient<'a, T>
where
    T: DropboxHttpTransport + ?Sized,
{
    pub fn new(config: DropboxClientConfig, transport: &'a T) -> Self {
        Self { config, transport }
    }

    pub fn exchange_authorization_code(
        &self,
        authorization_code: &str,
        code_verifier: &str,
        redirect_uri: Option<&str>,
    ) -> Result<DropboxToken, DropboxClientError> {
        let mut form = form_urlencoded::Serializer::new(String::new());
        form.append_pair("grant_type", "authorization_code")
            .append_pair("client_id", &self.config.client_id)
            .append_pair("code", authorization_code)
            .append_pair("code_verifier", code_verifier);
        if let Some(redirect_uri) = redirect_uri {
            form.append_pair("redirect_uri", redirect_uri);
        }

        self.post_token_form(form.finish())
    }

    pub fn refresh_access_token(
        &self,
        refresh_token: &str,
    ) -> Result<DropboxToken, DropboxClientError> {
        let mut form = form_urlencoded::Serializer::new(String::new());
        form.append_pair("grant_type", "refresh_token")
            .append_pair("client_id", &self.config.client_id)
            .append_pair("refresh_token", refresh_token);

        self.post_token_form(form.finish())
    }

    pub fn list_folder_recursive(
        &self,
        access_token: &str,
        path: &str,
    ) -> Result<Vec<DropboxEntry>, DropboxClientError> {
        let first_page = self.post_json(
            access_token,
            DROPBOX_LIST_FOLDER_URL,
            serde_json::json!({
                "path": path,
                "recursive": true,
                "include_deleted": false
            })
            .to_string(),
        )?;

        let mut entries = first_page.entries;
        let mut cursor = first_page.cursor;
        let mut has_more = first_page.has_more;

        while has_more {
            let next_page = self.post_json(
                access_token,
                DROPBOX_LIST_FOLDER_CONTINUE_URL,
                serde_json::json!({ "cursor": cursor }).to_string(),
            )?;
            entries.extend(next_page.entries);
            cursor = next_page.cursor;
            has_more = next_page.has_more;
        }

        Ok(entries.into_iter().map(DropboxEntry::from).collect())
    }

    fn post_token_form(&self, body: String) -> Result<DropboxToken, DropboxClientError> {
        let response = self.transport.send(DropboxHttpRequest {
            method: "POST".into(),
            url: DROPBOX_TOKEN_URL.into(),
            headers: vec![(
                "Content-Type".into(),
                "application/x-www-form-urlencoded".into(),
            )],
            body,
        })?;

        if response.status != 200 {
            return Err(DropboxClientError::UnexpectedStatus(
                response.status,
                response.body,
            ));
        }

        let body: DropboxTokenResponse = serde_json::from_str(&response.body)
            .map_err(|error| DropboxClientError::InvalidResponse(error.to_string()))?;
        Ok(DropboxToken {
            access_token: body.access_token,
            refresh_token: body.refresh_token,
            expires_in: body.expires_in,
            account_id: body.account_id,
        })
    }

    fn post_json(
        &self,
        access_token: &str,
        url: &str,
        body: String,
    ) -> Result<DropboxListFolderResponse, DropboxClientError> {
        let response = self.transport.send(DropboxHttpRequest {
            method: "POST".into(),
            url: url.into(),
            headers: vec![
                ("Authorization".into(), format!("Bearer {access_token}")),
                ("Content-Type".into(), "application/json".into()),
            ],
            body,
        })?;

        if response.status != 200 {
            return Err(DropboxClientError::UnexpectedStatus(
                response.status,
                response.body,
            ));
        }

        serde_json::from_str(&response.body)
            .map_err(|error| DropboxClientError::InvalidResponse(error.to_string()))
    }
}

#[derive(Deserialize)]
struct DropboxTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64,
    account_id: Option<String>,
}

#[derive(Deserialize)]
struct DropboxListFolderResponse {
    entries: Vec<DropboxEntryResponse>,
    cursor: String,
    has_more: bool,
}

#[derive(Deserialize)]
struct DropboxEntryResponse {
    #[serde(rename = ".tag")]
    tag: DropboxEntryTagResponse,
    name: String,
    id: String,
    path_lower: Option<String>,
    path_display: Option<String>,
    server_modified: Option<String>,
    rev: Option<String>,
    size: Option<u64>,
    content_hash: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum DropboxEntryTagResponse {
    File,
    Folder,
}

impl From<DropboxEntryResponse> for DropboxEntry {
    fn from(response: DropboxEntryResponse) -> Self {
        Self {
            tag: match response.tag {
                DropboxEntryTagResponse::File => DropboxEntryTag::File,
                DropboxEntryTagResponse::Folder => DropboxEntryTag::Folder,
            },
            name: response.name,
            id: response.id,
            path_lower: response.path_lower,
            path_display: response.path_display,
            server_modified: response.server_modified,
            rev: response.rev,
            size: response.size,
            content_hash: response.content_hash,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::collections::VecDeque;

    use super::{
        DropboxApiClient, DropboxClientConfig, DropboxEntryTag, DropboxHttpRequest,
        DropboxHttpResponse, DropboxHttpTransport, ReqwestDropboxHttpTransport,
    };

    #[derive(Default)]
    struct FakeTransport {
        requests: RefCell<Vec<DropboxHttpRequest>>,
    }

    impl DropboxHttpTransport for FakeTransport {
        fn send(
            &self,
            request: DropboxHttpRequest,
        ) -> Result<DropboxHttpResponse, super::DropboxClientError> {
            self.requests.borrow_mut().push(request);
            Ok(DropboxHttpResponse {
                status: 200,
                body: r#"{
                    "access_token": "access-123",
                    "refresh_token": "refresh-123",
                    "expires_in": 14400,
                    "account_id": "dbid:123",
                    "token_type": "bearer"
                }"#
                .into(),
            })
        }
    }

    struct QueuedTransport {
        requests: RefCell<Vec<DropboxHttpRequest>>,
        responses: RefCell<VecDeque<DropboxHttpResponse>>,
    }

    impl QueuedTransport {
        fn with(responses: Vec<DropboxHttpResponse>) -> Self {
            Self {
                requests: RefCell::new(vec![]),
                responses: RefCell::new(VecDeque::from(responses)),
            }
        }
    }

    impl DropboxHttpTransport for QueuedTransport {
        fn send(
            &self,
            request: DropboxHttpRequest,
        ) -> Result<DropboxHttpResponse, super::DropboxClientError> {
            self.requests.borrow_mut().push(request);
            Ok(self.responses.borrow_mut().pop_front().unwrap())
        }
    }

    #[test]
    fn exchanges_an_authorization_code_for_a_dropbox_token() {
        let transport = FakeTransport::default();
        let client = DropboxApiClient::new(
            DropboxClientConfig {
                client_id: "app-key".into(),
            },
            &transport,
        );

        let token = client
            .exchange_authorization_code(
                "auth-code",
                "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._~",
                Some("muzo://dropbox-auth"),
            )
            .expect("token exchange should succeed");

        assert_eq!(token.access_token, "access-123");
        assert_eq!(token.refresh_token.as_deref(), Some("refresh-123"));
        assert_eq!(token.expires_in, 14400);
        assert_eq!(token.account_id.as_deref(), Some("dbid:123"));

        let requests = transport.requests.borrow();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, "POST");
        assert_eq!(requests[0].url, "https://api.dropboxapi.com/oauth2/token");
        assert_eq!(
            requests[0].headers,
            vec![(
                "Content-Type".to_string(),
                "application/x-www-form-urlencoded".to_string()
            )]
        );
        let form: HashMap<_, _> =
            url::form_urlencoded::parse(requests[0].body.as_bytes()).collect();
        assert_eq!(
            form.get("grant_type").map(|value| value.as_ref()),
            Some("authorization_code")
        );
        assert_eq!(
            form.get("client_id").map(|value| value.as_ref()),
            Some("app-key")
        );
        assert_eq!(
            form.get("code").map(|value| value.as_ref()),
            Some("auth-code")
        );
        assert_eq!(
            form.get("code_verifier").map(|value| value.as_ref()),
            Some("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._~")
        );
        assert_eq!(
            form.get("redirect_uri").map(|value| value.as_ref()),
            Some("muzo://dropbox-auth")
        );
    }

    #[test]
    fn refreshes_a_dropbox_access_token() {
        let transport = FakeTransport::default();
        let client = DropboxApiClient::new(
            DropboxClientConfig {
                client_id: "app-key".into(),
            },
            &transport,
        );

        let token = client
            .refresh_access_token("refresh-123")
            .expect("refresh should succeed");

        assert_eq!(token.access_token, "access-123");

        let requests = transport.requests.borrow();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, "POST");
        assert_eq!(requests[0].url, "https://api.dropboxapi.com/oauth2/token");
        let form: HashMap<_, _> =
            url::form_urlencoded::parse(requests[0].body.as_bytes()).collect();
        assert_eq!(
            form.get("grant_type").map(|value| value.as_ref()),
            Some("refresh_token")
        );
        assert_eq!(
            form.get("client_id").map(|value| value.as_ref()),
            Some("app-key")
        );
        assert_eq!(
            form.get("refresh_token").map(|value| value.as_ref()),
            Some("refresh-123")
        );
    }

    #[test]
    fn lists_a_dropbox_folder_recursively_across_pages() {
        let transport = QueuedTransport::with(vec![
            DropboxHttpResponse {
                status: 200,
                body: r#"{
                    "entries": [
                        {
                            ".tag": "folder",
                            "name": "Album",
                            "path_lower": "/music/album",
                            "path_display": "/Music/Album",
                            "id": "id:folder"
                        }
                    ],
                    "cursor": "cursor-1",
                    "has_more": true
                }"#
                .into(),
            },
            DropboxHttpResponse {
                status: 200,
                body: r#"{
                    "entries": [
                        {
                            ".tag": "file",
                            "name": "Song.mp3",
                            "path_lower": "/music/album/song.mp3",
                            "path_display": "/Music/Album/Song.mp3",
                            "id": "id:file",
                            "server_modified": "2026-06-25T10:00:00Z",
                            "rev": "rev-1",
                            "size": 12345,
                            "content_hash": "hash-1"
                        }
                    ],
                    "cursor": "cursor-2",
                    "has_more": false
                }"#
                .into(),
            },
        ]);
        let client = DropboxApiClient::new(
            DropboxClientConfig {
                client_id: "app-key".into(),
            },
            &transport,
        );

        let entries = client
            .list_folder_recursive("access-123", "/Music")
            .expect("folder listing should succeed");

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].tag, DropboxEntryTag::Folder);
        assert_eq!(entries[0].path_display.as_deref(), Some("/Music/Album"));
        assert_eq!(entries[1].tag, DropboxEntryTag::File);
        assert_eq!(entries[1].name, "Song.mp3");
        assert_eq!(entries[1].size, Some(12345));

        let requests = transport.requests.borrow();
        assert_eq!(requests.len(), 2);
        assert_eq!(
            requests[0].url,
            "https://api.dropboxapi.com/2/files/list_folder"
        );
        assert_eq!(
            requests[0].headers,
            vec![
                ("Authorization".to_string(), "Bearer access-123".to_string()),
                ("Content-Type".to_string(), "application/json".to_string())
            ]
        );
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&requests[0].body).unwrap(),
            serde_json::json!({
                "path": "/Music",
                "recursive": true,
                "include_deleted": false
            })
        );
        assert_eq!(
            requests[1].url,
            "https://api.dropboxapi.com/2/files/list_folder/continue"
        );
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&requests[1].body).unwrap(),
            serde_json::json!({ "cursor": "cursor-1" })
        );
    }

    #[test]
    fn reqwest_transport_sends_the_http_request_and_returns_the_response() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = Vec::new();
            let mut buffer = [0_u8; 512];
            loop {
                let read = std::io::Read::read(&mut stream, &mut buffer).unwrap();
                request.extend_from_slice(&buffer[..read]);
                if request.windows(4).any(|window| window == b"\r\n\r\n")
                    && String::from_utf8_lossy(&request).contains("{\"ok\":true}")
                {
                    break;
                }
            }
            let request = String::from_utf8(request).unwrap();
            assert!(request.starts_with("POST /token HTTP/1.1"));
            assert!(request.contains("authorization: Bearer access-123"));
            assert!(request.contains("content-type: application/json"));
            assert!(request.contains("{\"ok\":true}"));
            std::io::Write::write_all(
                &mut stream,
                b"HTTP/1.1 200 OK\r\nContent-Length: 14\r\nConnection: close\r\n\r\n{\"done\":true}\n",
            )
            .unwrap();
        });
        let transport = ReqwestDropboxHttpTransport::new();

        let response = transport
            .send(DropboxHttpRequest {
                method: "POST".into(),
                url: format!("http://{address}/token"),
                headers: vec![
                    ("Authorization".into(), "Bearer access-123".into()),
                    ("Content-Type".into(), "application/json".into()),
                ],
                body: r#"{"ok":true}"#.into(),
            })
            .expect("transport should return the server response");

        handle.join().unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.body, "{\"done\":true}\n");
    }
}
