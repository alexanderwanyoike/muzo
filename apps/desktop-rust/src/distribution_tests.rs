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
