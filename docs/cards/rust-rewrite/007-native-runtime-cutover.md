# 007: Cut over to the Rust desktop

- [x] Run desktop development, builds, tests and packaging from the Rust workspace.
- [x] Remove the Electron workspace and its build and distribution jobs.
- [ ] Verify the native installer matrix from the full checkout on Linux, Windows and both Macs.
- [ ] Verify the installed upgrades from the published Muzo release.
- [ ] Verify the local, Dropbox, playlist and mini-player journeys on each target.

[ADR 0011](../../adr/0011-rust-iced-desktop.md) defines the runtime cutover.
[Card 006](006-distribution.md) describes the installer and release contracts.
Use the published Electron installers to verify upgrades against the shipped
runtime and profile formats. Keep the desktop installers, update feed and data
identity compatible with the previous release.
