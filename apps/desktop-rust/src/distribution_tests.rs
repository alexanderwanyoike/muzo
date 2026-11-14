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
