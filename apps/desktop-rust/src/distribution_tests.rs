use crate::distribution::{Target, filename};

#[test]
fn it_keeps_the_existing_release_filenames_for_all_targets() {
    assert_eq!(
        filename("0.0.6", Target::LinuxX64),
        "Muzo-0.0.6-linux-x64.AppImage"
    );
    assert_eq!(
        filename("0.0.6", Target::WindowsX64),
        "Muzo-0.0.6-windows-x64-setup.exe"
    );
    assert_eq!(
        filename("0.0.6", Target::MacArm64),
        "Muzo-0.0.6-macos-arm64.dmg"
    );
    assert_eq!(
        filename("0.0.6", Target::MacX64),
        "Muzo-0.0.6-macos-x64.dmg"
    );
}

#[test]
fn it_selects_the_installer_and_checksum_for_the_current_target() {
    use crate::distribution::{Release, update_for};
    let release: Release = serde_json::from_str(r#"{
       "schemaVersion":1,"tag":"v0.0.7","version":"0.0.7","sourceCommit":"0000000000000000000000000000000000000000",
       "artifacts":[
         {"name":"Muzo-0.0.7-linux-x64.AppImage","platform":"linux","arch":"x64","size":12,"sha256":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"},
         {"name":"Muzo-0.0.7-windows-x64-setup.exe","platform":"windows","arch":"x64","size":42,"sha256":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}
       ]
    }"#).unwrap();
    let update = update_for("0.0.6", Target::WindowsX64, release)
        .unwrap()
        .unwrap();
    assert_eq!(update.asset.size, 42);
    assert_eq!(update.asset.name, "Muzo-0.0.7-windows-x64-setup.exe");
    assert!(
        update
            .download_url()
            .unwrap()
            .as_str()
            .ends_with("/v0.0.7/Muzo-0.0.7-windows-x64-setup.exe")
    );
}

#[test]
fn it_rejects_a_download_modified_before_installation() {
    use crate::distribution::{Artifact, Update};
    use sha2::{Digest, Sha256};

    let installer = tempfile::NamedTempFile::new().unwrap();
    let original = b"muzo";
    std::fs::write(installer.path(), original).unwrap();
    let update = Update {
        version: "0.0.7".parse().unwrap(),
        asset: Artifact {
            name: "Muzo-0.0.7-linux-x64.AppImage".into(),
            platform: "linux".into(),
            arch: "x64".into(),
            size: original.len() as u64,
            sha256: format!("{:x}", Sha256::digest(original)),
        },
    };
    update.verify(installer.path()).unwrap();
    std::fs::write(installer.path(), b"bad!").unwrap();
    assert!(update.verify(installer.path()).is_err());
}

#[test]
fn it_rejects_an_update_with_mismatched_platform_metadata() {
    use crate::distribution::{Artifact, Release, update_for};
    let release = Release {
        schema_version: 1,
        tag: "v0.0.7".into(),
        version: "0.0.7".into(),
        source_commit: "a".repeat(40),
        artifacts: vec![Artifact {
            name: filename("0.0.7", Target::LinuxX64),
            platform: "windows".into(),
            arch: "x64".into(),
            size: 12,
            sha256: "a".repeat(64),
        }],
    };

    assert!(update_for("0.0.6", Target::LinuxX64, release).is_err());
}
