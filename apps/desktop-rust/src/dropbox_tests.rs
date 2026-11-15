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

#[test]
fn it_indexes_remote_audio_and_preserves_an_existing_track_id_on_resync() {
    use crate::{catalogue::Catalogue, dropbox::Entry};
    use std::path::Path;

    let root = tempfile::tempdir().unwrap();
    let mut catalogue = Catalogue::open(&root.path().join("catalogue.sqlite"), None).unwrap();
    catalogue
        .save_dropbox_account("account", "app-key")
        .unwrap();
    let library = catalogue.add_dropbox("account", "/Music", "Music").unwrap();
    let audio: Entry = serde_json::from_str(
        r#"{".tag":"file","name":"A.mp3","path_display":"/Music/A.mp3","size":48,"rev":"1"}"#,
    )
    .unwrap();
    catalogue
        .sync_dropbox(&library.id, std::slice::from_ref(&audio))
        .unwrap();
    let track = catalogue.tracks(&library.id).unwrap().remove(0);
    assert_eq!(track.path, Path::new("/Music/A.mp3"));
    assert_eq!(
        catalogue
            .dropbox_account_for(&library.id)
            .unwrap()
            .as_deref(),
        Some("account")
    );
    catalogue.sync_dropbox(&library.id, &[audio]).unwrap();
    assert_eq!(catalogue.tracks(&library.id).unwrap()[0].id, track.id);
}

#[test]
fn it_does_not_replace_cached_metadata_when_remote_audio_is_reindexed() {
    use crate::{catalogue::Catalogue, dropbox::Entry};
    let root = tempfile::tempdir().unwrap();
    let mut catalogue = Catalogue::open(&root.path().join("catalogue.sqlite"), None).unwrap();
    catalogue
        .save_dropbox_account("account", "app-key")
        .unwrap();
    let library = catalogue.add_dropbox("account", "/Music", "Music").unwrap();
    let audio: Entry = serde_json::from_str(
        r#"{".tag":"file","name":"A.mp3","path_display":"/Music/A.mp3","size":48,"rev":"1"}"#,
    )
    .unwrap();
    catalogue
        .sync_dropbox(&library.id, std::slice::from_ref(&audio))
        .unwrap();
    let indexed = catalogue.tracks(&library.id).unwrap().remove(0);
    catalogue
        .cache_dropbox_metadata(&indexed.id, Some("Tagged"), Some("Artist"), Some("Record"))
        .unwrap();
    catalogue.sync_dropbox(&library.id, &[audio]).unwrap();
    let indexed = catalogue.tracks(&library.id).unwrap().remove(0);
    assert_eq!(indexed.title, "Tagged");
    assert_eq!(indexed.artist, "Artist");
}

#[test]
fn it_imports_an_electron_refresh_token_once_and_reuses_the_native_credential() {
    use crate::{catalogue::Catalogue, dropbox::Credentials};
    let folder = tempfile::tempdir().unwrap();
    let catalogue = Catalogue::open(&folder.path().join("muzo.sqlite"), None).unwrap();
    catalogue
        .save_dropbox_account("account", "app-key")
        .unwrap();
    let database = rusqlite::Connection::open(folder.path().join("muzo.sqlite")).unwrap();
    database
        .execute(
            "UPDATE dropbox_accounts SET encrypted_refresh_token='encrypted'",
            [],
        )
        .unwrap();
    let remote = std::cell::RefCell::new(None);
    let imported = Credentials::migrate_with(
        &catalogue,
        "account",
        || Ok(remote.borrow().clone()),
        |encrypted| {
            assert_eq!(encrypted, "encrypted");
            Ok("fixture-refresh-token".to_owned())
        },
        |token| {
            remote.replace(Some(token.into()));
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(imported.as_deref(), Some("fixture-refresh-token"));

    let reopened = Credentials::migrate_with(
        &catalogue,
        "account",
        || Ok(remote.borrow().clone()),
        |_| panic!("already imported"),
        |_| panic!("already imported"),
    )
    .unwrap();
    assert_eq!(reopened, imported);
}

#[test]
fn it_persists_the_native_credential_store_for_each_dropbox_account() {
    use crate::catalogue::Catalogue;
    let root = tempfile::tempdir().unwrap();
    let database = root.path().join("muzo-rust.sqlite");
    let catalogue = Catalogue::open(&database, None).unwrap();
    catalogue.save_dropbox_account("first", "app-key").unwrap();
    catalogue.save_dropbox_account("second", "app-key").unwrap();
    catalogue
        .set_dropbox_credential_store("first", "kwallet")
        .unwrap();
    catalogue
        .set_dropbox_credential_store("second", "keyring")
        .unwrap();
    drop(catalogue);

    let catalogue = Catalogue::open(&database, None).unwrap();
    assert_eq!(
        catalogue
            .dropbox_credential_store("first")
            .unwrap()
            .as_deref(),
        Some("kwallet")
    );
    assert_eq!(
        catalogue
            .dropbox_credential_store("second")
            .unwrap()
            .as_deref(),
        Some("keyring")
    );
    catalogue.disconnect_dropbox("first").unwrap();
    assert_eq!(catalogue.dropbox_credential_store("first").unwrap(), None);
}

#[test]
fn it_invalidates_artwork_when_dropbox_replaces_a_file_with_the_same_size() {
    use crate::{artwork_cache::ArtworkCache, catalogue::Catalogue};
    let root = tempfile::tempdir().unwrap();
    let mut catalogue = Catalogue::open(&root.path().join("muzo-rust.sqlite"), None).unwrap();
    catalogue.save_dropbox_account("account", "app").unwrap();
    let library = catalogue.add_dropbox("account", "/Music", "Music").unwrap();
    let mut entry: crate::dropbox::Entry = serde_json::from_str(
        r#"{".tag":"file","name":"Track.mp3","path_display":"/Music/Track.mp3","size":48,"rev":"initial"}"#,
    ).unwrap();
    catalogue
        .sync_dropbox(&library.id, std::slice::from_ref(&entry))
        .unwrap();
    let id = catalogue.tracks(&library.id).unwrap().remove(0).id;
    let cache = ArtworkCache::new(root.path().join("covers"));
    let first = catalogue.artwork_revision(&library.id, &id).unwrap();
    assert!(
        cache
            .get_or_read(&first, || Some(
                include_bytes!("../assets/icons/muzo.png").to_vec()
            ))
            .is_some()
    );

    entry.rev = Some("updated".into());
    catalogue.sync_dropbox(&library.id, &[entry]).unwrap();
    let latest = catalogue.artwork_revision(&library.id, &id).unwrap();
    assert!(cache.get_or_read(&latest, || None).is_none());
}

#[test]
fn it_rolls_back_a_dropbox_source_if_the_initial_track_index_cannot_be_written() {
    use crate::{catalogue::Catalogue, dropbox::Entry};

    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("catalogue.sqlite");
    let mut catalogue = Catalogue::open(&path, None).unwrap();
    catalogue
        .save_dropbox_account("account", "app-key")
        .unwrap();
    let database = rusqlite::Connection::open(&path).unwrap();
    database
        .execute_batch(
            "CREATE TRIGGER fail_initial_track BEFORE INSERT ON tracks
             WHEN NEW.title = 'Broken'
             BEGIN SELECT RAISE(ABORT, 'index write failed'); END;",
        )
        .unwrap();
    let valid: Entry =
        serde_json::from_str(r#"{".tag":"file","name":"A.mp3","path_display":"/Music/A.mp3"}"#)
            .unwrap();
    let invalid: Entry = serde_json::from_str(
        r#"{".tag":"file","name":"Broken.mp3","path_display":"/Music/Broken.mp3"}"#,
    )
    .unwrap();

    let error = catalogue
        .add_dropbox_indexed("account", "/Music", "Music", &[valid.clone(), invalid])
        .unwrap_err();
    assert!(error.to_string().contains("index write failed"));
    let reopened = Catalogue::open(&path, None).unwrap();
    assert!(reopened.libraries().unwrap().is_empty());
    assert!(
        reopened
            .dropbox_libraries_for("account")
            .unwrap()
            .is_empty()
    );
    let library = catalogue
        .add_dropbox_indexed("account", "/Music", "Music", &[valid])
        .unwrap();
    assert_eq!(reopened.tracks(&library.id).unwrap().len(), 1);
}

#[test]
fn it_keeps_a_dropbox_folder_without_supported_audio_out_of_the_catalogue() {
    use crate::{catalogue::Catalogue, dropbox::Entry};

    let root = tempfile::tempdir().unwrap();
    let mut catalogue = Catalogue::open(&root.path().join("muzo.sqlite"), None).unwrap();
    catalogue.save_dropbox_account("account", "client").unwrap();
    let entries: Vec<Entry> = serde_json::from_str(
        r#"[{".tag":"folder","name":"Images","path_display":"/Music/Images"},
            {".tag":"file","name":"notes.txt","path_display":"/Music/notes.txt"}]"#,
    )
    .unwrap();

    let result = catalogue.add_dropbox_indexed("account", "/Music", "Music", &entries);

    assert!(
        result.is_err(),
        "a Dropbox library with no supported tracks was added"
    );
    assert!(
        catalogue
            .dropbox_libraries_for("account")
            .unwrap()
            .is_empty()
    );
    assert!(catalogue.libraries().unwrap().is_empty());
}
