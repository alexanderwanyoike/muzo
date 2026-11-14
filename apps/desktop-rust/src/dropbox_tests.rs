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

#[test]
fn it_uses_offline_authorization_and_follows_the_dropbox_folder_cursor() {
    use std::{
        io::{BufRead, BufReader, Read, Write},
        net::TcpListener,
    };

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        let replies = [
            (
                "POST /oauth2/token ",
                r#"{"access_token":"fixture-token","refresh_token":"fixture-refresh","expires_in":300,"account_id":"account"}"#,
            ),
            (
                "POST /2/files/list_folder ",
                r#"{"entries":[{".tag":"folder","name":"Recordings","path_display":"/Recordings"}],"cursor":"next","has_more":true}"#,
            ),
            (
                "POST /2/files/list_folder/continue ",
                r#"{"entries":[{".tag":"folder","name":"Archive","path_display":"/Archive"}],"cursor":"end","has_more":false}"#,
            ),
        ];
        let mut requests = Vec::new();
        for (route, response) in replies {
            let (mut connection, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(&mut connection);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            assert!(line.starts_with(route));
            let mut content_length = 0;
            loop {
                line.clear();
                reader.read_line(&mut line).unwrap();
                if line == "\r\n" {
                    break;
                }
                if let Some(value) = line.to_lowercase().strip_prefix("content-length:") {
                    content_length = value.trim().parse().unwrap();
                }
            }
            let mut body = vec![0; content_length];
            reader.read_exact(&mut body).unwrap();
            requests.push(String::from_utf8(body).unwrap());
            write!(connection, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}", response.len()).unwrap();
        }
        requests
    });

    let client = crate::dropbox::Client::for_endpoint(&address).unwrap();
    let request = Authorization::new("muzo-test-app").unwrap();
    let token = client.exchange(&request, "fixture-code").unwrap();
    let paths = client.list(&token.access_token, "/", false).unwrap();
    assert_eq!(
        paths
            .iter()
            .filter_map(|entry| entry.path())
            .collect::<Vec<_>>(),
        ["/Recordings", "/Archive"]
    );
    let requests = server.join().unwrap();
    assert!(requests[0].contains("grant_type=authorization_code"));
    assert!(requests[0].contains(&format!("code_verifier={}", request.verifier())));
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&requests[1]).unwrap()["path"],
        ""
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&requests[2]).unwrap()["cursor"],
        "next"
    );
}
