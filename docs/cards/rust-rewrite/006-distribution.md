# 006: Package native Rust desktop builds

- [ ] Produce and validate Linux AppImage, Windows NSIS and both macOS DMGs.
- [ ] Retain established installer filenames and release manifest compatibility.
- [ ] Exercise installed upgrades and explicit download and restart.
- [ ] Preserve catalogue and preferences on upgrade.

All installers use the icons and licence notices under `apps/desktop-rust/assets`.
Cargo Packager builds the AppImage and macOS app bundle. The Linux release job
builds and opens the AppImage on Ubuntu 22.04, matching the previous release's
glibc baseline and following [AppImage's build-host guidance](https://docs.appimage.org/introduction/concepts.html#build-on-old-systems-run-on-newer-systems).
The Linux build host also needs the X11 keyboard library (`libxkbcommon-x11-0`
on Ubuntu). It is copied into the AppImage for Iced's X11 startup through
[Cargo Packager's AppImage library configuration](https://docs.crabnebula.dev/packager/configuration/).
The macOS job stages the bundle with an Applications link, then creates a
read-only DMG with
`hdiutil`, following [Apple-Actions' disk image example](https://github.com/Apple-Actions/Example-macOS#running-locally).
The Windows job keeps the established application ID and NSIS install options
in `apps/desktop-rust/windows-installer.json`. Its build-time NSIS tool receives
the Rust executable and notices as a prepackaged payload, following the
[electron-builder NSIS upgrade contract](https://www.electron.build/v26/docs/nsis/#guid-vs-application-name).
The release filename and manifest contracts remain shared with the website.

The Rust desktop installers workflow prepares the release draft for annotated
version tags on `main`. It validates the root version, the Rust workspace
version, Cargo and Packager before starting the four installer jobs. The draft
uses the assembled native installer set and its verified checksums.
The upgrade fixtures verify installs using the published v0.0.6 installers.
