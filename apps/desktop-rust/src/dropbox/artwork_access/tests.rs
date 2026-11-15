use std::{
    any::Any,
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
    process::Command,
};

use keyring::{Credential, credential::CredentialBuilderApi};

use super::*;

const ACCOUNT: &str = "fixture-account";
const CLIENT: &str = "fixture-client";
const REFRESH_TOKEN: &str = "fixture-refresh";

struct RefreshCredential;

impl CredentialBuilderApi for RefreshCredential {
    fn build(
        &self,
        target: Option<&str>,
        service: &str,
        user: &str,
    ) -> keyring::Result<Box<Credential>> {
        assert_eq!(service, "muzo.dropbox.refresh");
        assert_eq!(user, ACCOUNT);
        let credential =
            keyring::mock::default_credential_builder().build(target, service, user)?;
        credential.set_password(REFRESH_TOKEN)?;
        Ok(credential)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[test]
fn it_reads_artwork_with_a_new_token_after_dropbox_rejects_the_cached_token() {
    const ISOLATED: &str = "MUZO_ARTWORK_CREDENTIAL_TEST";
    if std::env::var_os(ISOLATED).is_none() {
        let child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "dropbox::artwork_access::tests::it_reads_artwork_with_a_new_token_after_dropbox_rejects_the_cached_token"])
            .env(ISOLATED, "1")
            .output()
            .unwrap();
        assert!(
            child.status.success(),
            "{}",
            String::from_utf8_lossy(&child.stdout)
        );
        return;
    }

    keyring::set_default_credential_builder(Box::new(RefreshCredential));
    let fixture = tempfile::tempdir().unwrap();
    let mut catalogue = Catalogue::open(&fixture.path().join("catalogue.sqlite"), None).unwrap();
    catalogue
        .save_native_dropbox_account(ACCOUNT, CLIENT, "keyring")
        .unwrap();
    let library = catalogue.add_dropbox(ACCOUNT, "/Music", "Music").unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let (route, headers, _) = request(&stream);
        assert!(route.starts_with("GET /2/files/download "));
        assert!(headers.contains("authorization: bearer rejected-token"));
        reply(&mut stream, "401 Unauthorized", "", &[]);

        let (mut stream, _) = listener.accept().unwrap();
        let (route, _, body) = request(&stream);
        assert!(
            route.starts_with("POST /oauth2/token "),
            "unexpected request {route}"
        );
        assert!(body.contains("grant_type=refresh_token"));
        assert!(body.contains(&format!("client_id={CLIENT}")));
        assert!(body.contains(&format!("refresh_token={REFRESH_TOKEN}")));
        reply(
            &mut stream,
            "200 OK",
            "Content-Type: application/json\r\n",
            br#"{"access_token":"renewed-token","expires_in":3600}"#,
        );

        let (mut stream, _) = listener.accept().unwrap();
        let (route, headers, _) = request(&stream);
        assert!(route.starts_with("GET /2/files/download "));
        assert!(headers.contains("authorization: bearer renewed-token"));
        assert!(headers.contains("range: bytes=0-65535"));
        let audio = include_bytes!("../../../tests/fixtures/embedded-cover.mp3");
        reply(
            &mut stream,
            "206 Partial Content",
            &format!(
                "Content-Range: bytes 0-{}/{}\r\n",
                audio.len() - 1,
                audio.len()
            ),
            audio,
        );
    });

    let access = ArtworkAccess(Mutex::new(Authorizations {
        client: Some(Arc::new(Client::for_endpoint(&endpoint).unwrap())),
        tokens: HashMap::from([(
            ACCOUNT.into(),
            Access {
                key: CLIENT.into(),
                token: "rejected-token".into(),
                expires: Instant::now() + Duration::from_secs(60),
            },
        )]),
    }));
    let read = || access.read(&catalogue, &library.id, "/Music/Track.mp3");
    assert!(matches!(read(), Err(DropboxError::Reconnect)));
    let cover = read().expect("the second read should use renewed credentials");
    assert!(cover.is_some_and(|bytes| image::load_from_memory(&bytes).is_ok()));
    server.join().unwrap();
}

fn request(stream: &std::net::TcpStream) -> (String, String, String) {
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    let mut request = BufReader::new(stream);
    let mut route = String::new();
    request.read_line(&mut route).unwrap();
    let mut headers = String::new();
    loop {
        let mut line = String::new();
        request.read_line(&mut line).unwrap();
        if line == "\r\n" || line.is_empty() {
            break;
        }
        headers.push_str(&line.to_ascii_lowercase());
    }
    let length = headers
        .lines()
        .find_map(|line| line.strip_prefix("content-length: "));
    let mut body = vec![0; length.map_or(0, |value| value.parse().unwrap())];
    request.read_exact(&mut body).unwrap();
    (route, headers, String::from_utf8(body).unwrap())
}

fn reply(stream: &mut std::net::TcpStream, status: &str, headers: &str, body: &[u8]) {
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n{headers}\r\n",
        body.len()
    )
    .unwrap();
    stream.write_all(body).unwrap();
}
