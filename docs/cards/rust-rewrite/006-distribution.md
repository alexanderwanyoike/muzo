# 006: Package native Rust desktop builds

- [ ] Produce and validate Linux AppImage, Windows NSIS and both macOS DMGs.
- [ ] Retain established installer filenames and release manifest compatibility.
- [ ] Exercise installed upgrades and explicit download and restart.
- [ ] Preserve catalogue and preferences on upgrade.

Cargo Packager builds the AppImage and macOS app bundle. The macOS job stages
the bundle with an Applications link, then creates a read-only DMG with
`hdiutil`, following [Apple-Actions' disk image example](https://github.com/Apple-Actions/Example-macOS#running-locally).
Windows uses the existing
electron-builder NSIS configuration to package the Rust executable and retain
the application and installer identity across an upgrade. The release filename
and manifest contracts remain shared with the website.
