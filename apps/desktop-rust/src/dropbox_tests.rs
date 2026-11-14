use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};

use crate::dropbox::Authorization;

#[test]
fn it_starts_offline_pkce_authorization_without_a_client_secret() {
    let request = Authorization::new("muzo-test-app").unwrap();
    let url = request.url().unwrap();
    let query: std::collections::HashMap<_, _> = url.query_pairs().collect();
    assert_eq!(url.host_str(), Some("www.dropbox.com"));
    assert_eq!(query["client_id"], "muzo-test-app");
    assert_eq!(query["token_access_type"], "offline");
    assert_eq!(query["response_type"], "code");
    assert_eq!(query["code_challenge_method"], "S256");
    assert_eq!(
        query["code_challenge"],
        URL_SAFE_NO_PAD.encode(Sha256::digest(request.verifier()))
    );
    assert!(!query.contains_key("client_secret"));
    assert!(!query.contains_key("code_verifier"));
    assert_ne!(
        request.verifier(),
        Authorization::new("muzo-test-app").unwrap().verifier()
    );
}
