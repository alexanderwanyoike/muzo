- Sprint: 02
- Type: Feature
- Status: In review
- Depends on: 025 (PR #52)

# Card 026 - Native app icons

Give Muzo a consistent app identity on Windows, Linux and macOS. The user
rejected the metallic design and requested blue, black and flat. Ship the
blue waveform bars on a black rounded tile, with transparent corners. The bars
form two peaks and a central trough that suggest an M, following the user's
follow-up request to replace the continuous M with a waveform style.

## Acceptance criteria

- [x] Commit the master PNG, Windows ICO and macOS ICNS assets.
- [x] Include small Windows sizes and macOS artwork through 1024 pixels.
- [x] Use the native window icon in development and packaged builds.
- [x] Set the Muzo development Dock icon; retain the native bundle icon when packaged.
- [x] Configure Forge icons and the Linux DEB launcher icon.
- [x] Provide reproducible Yarn exports and preserve converter licensing/provenance.
- [x] Decode the real platform image and create a native window in integration tests.
- [x] Add Windows/macOS native icon tests and packaging to CI.
- [x] Complete ordered checks and two review cycles (285 tests).
- [ ] Verify native platform CI.

## Established pattern

Follow Electron Forge's MIT-licensed custom-icon guidance:
https://www.electronforge.io/guides/create-and-add-icons
Use its extension-free packager icon path with PNG/ICO/ICNS siblings, and supply
PNG for the DEB launcher. Runtime assets remain in the root assets directory,
which Forge already includes, avoiding a duplicate build-time asset copy.
Electron BrowserWindow.icon and app.dock.setIcon supply the live native identity:
https://www.electronjs.org/docs/latest/api/browser-window
https://www.electronjs.org/docs/latest/api/dock

Use the established MIT-licensed png2icons 2.0.1 converter instead of implementing
binary formats: https://github.com/idesis-gmbh/png2icons.
The Windows-executable preset includes BMP small sizes and PNG larger sizes.
The generated PNG master is 1254 square, above Forge's recommended 1024 source.
No third-party artwork or converter implementation is copied. The converter
licence and exact built-in image generation prompts are in assets/icons.

## Verification

Red-first tests require the shipped native formats, platform-specific paths,
macOS Dock behavior and Forge/DEB configuration. Supplementary native integration
runs the real Electron decoder at 16, 32 and 256 pixels and creates a window,
using an isolated profile and Xvfb only on Linux. Windows/macOS CI also regenerates
the containers to detect drift and packages their native applications.
Manual OS-shell appearance still needs user testing on the target desktop.

Review evidence: [two cycles and final correction](../../reviews/2026-10-24-native-app-icons.md).

## Fresh-install native test fix

The first PR CI run 40093947210 passed native decoding and packaging on Windows
and macOS. Linux failed the existing artwork native test while the new icon
native test passed. Both workers logged "Downloading Electron binary...".
Electron 43's installed index.js downloads synchronously on first require, so
parallel fresh workers can race over the same installation. Prepare the binary
once before Vitest starts, using Electron's own entry point. Keep the native tests
and their assertions intact; watch mode uses the same preparation.
