use base64::Engine;
use sha2::Digest;
use url::Url;

const DROPBOX_AUTHORIZE_URL: &str = "https://www.dropbox.com/oauth2/authorize";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropboxPkceVerifier(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropboxState(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropboxOAuthAuthorizationRequest {
    pub client_id: String,
    pub redirect_uri: Option<String>,
    pub scopes: Vec<String>,
    pub state: DropboxState,
    pub code_verifier: DropboxPkceVerifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DropboxOAuthError {
    InvalidAuthorizeEndpoint(String),
}

pub fn build_dropbox_authorization_url(
    request: DropboxOAuthAuthorizationRequest,
) -> Result<Url, DropboxOAuthError> {
    let mut url = Url::parse(DROPBOX_AUTHORIZE_URL).map_err(|error| {
        DropboxOAuthError::InvalidAuthorizeEndpoint(format!(
            "invalid Dropbox authorize endpoint: {error}"
        ))
    })?;
    url.query_pairs_mut()
        .append_pair("client_id", &request.client_id)
        .append_pair("response_type", "code")
        .append_pair("token_access_type", "offline")
        .append_pair("code_challenge_method", "S256")
        .append_pair(
            "code_challenge",
            &pkce_s256_challenge(&request.code_verifier),
        )
        .append_pair("state", &request.state.0);

    if let Some(redirect_uri) = request.redirect_uri {
        url.query_pairs_mut()
            .append_pair("redirect_uri", &redirect_uri);
    }

    if !request.scopes.is_empty() {
        url.query_pairs_mut()
            .append_pair("scope", &request.scopes.join(" "));
    }

    Ok(url)
}

fn pkce_s256_challenge(verifier: &DropboxPkceVerifier) -> String {
    let digest = sha2::Sha256::digest(verifier.0.as_bytes());
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{
        build_dropbox_authorization_url, DropboxOAuthAuthorizationRequest, DropboxPkceVerifier,
        DropboxState,
    };

    #[test]
    fn builds_a_pkce_authorization_url_for_offline_dropbox_access() {
        let url = build_dropbox_authorization_url(DropboxOAuthAuthorizationRequest {
            client_id: "app-key".into(),
            redirect_uri: Some("muzo://dropbox-auth".into()),
            scopes: vec!["files.metadata.read".into()],
            state: DropboxState("state-123".into()),
            code_verifier: DropboxPkceVerifier(
                "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._~".into(),
            ),
        })
        .expect("authorization URL should be built");

        let query: HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(
            url.origin().ascii_serialization(),
            "https://www.dropbox.com"
        );
        assert_eq!(url.path(), "/oauth2/authorize");
        assert_eq!(query.get("client_id").map(String::as_str), Some("app-key"));
        assert_eq!(
            query.get("redirect_uri").map(String::as_str),
            Some("muzo://dropbox-auth")
        );
        assert_eq!(query.get("response_type").map(String::as_str), Some("code"));
        assert_eq!(
            query.get("token_access_type").map(String::as_str),
            Some("offline")
        );
        assert_eq!(
            query.get("code_challenge_method").map(String::as_str),
            Some("S256")
        );
        assert_eq!(
            query.get("code_challenge").map(String::as_str),
            Some("ImpiCd8pp4MveCNnbIS7-GXEtB0xF5HMIDoWqvGA5ig")
        );
        assert_eq!(query.get("state").map(String::as_str), Some("state-123"));
        assert_eq!(
            query.get("scope").map(String::as_str),
            Some("files.metadata.read")
        );
    }
}
