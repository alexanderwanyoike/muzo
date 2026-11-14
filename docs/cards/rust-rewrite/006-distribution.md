# 006: Package native Rust desktop builds

- [ ] Produce and validate Linux AppImage, Windows NSIS and both macOS DMGs.
- [ ] Retain established installer filenames and release manifest compatibility.
- [ ] Exercise installed upgrades and explicit download and restart.
- [ ] Preserve catalogue and preferences on upgrade.

Cargo Packager builds the installer formats from the Rust binary. The release
filename and manifest contracts remain shared with the website.
