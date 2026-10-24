- Sprint: 02
- Type: Feature
- Status: In review
- Depends on: 035 (#63), 038 (#65)

# Card 040 - Add an artwork-led compact player

The user requested a mini player inspired by a GTK4/libadwaita reference, with
artwork, transport controls and an optional playback queue in a small window.
This first slice uses the existing audio owner, materials and accent preferences.

## Acceptance criteria

- [x] Enter compact mode and return to the existing library view without reloading audio, resetting position or losing the selected playlist/source.
- [x] Show current embedded artwork, available title/artist/album metadata, real seek progress, previous/play/pause/next and volume.
- [x] Expand/collapse the real playback queue and select repeated tracks by their queue entry identity.
- [x] Keep playback errors readable and failed native transitions recoverable through typed Results.
- [x] Restore the original native window bounds, minimum size and maximized state; roll back rejected resize operations.
- [x] Preserve visible keyboard focus, move focus to the visible view, and retain native OS window controls.
- [x] Use the saved light/dark material and accent, with a quiet blurred-art backdrop.
- [x] Verify actual Electron geometry and IPC at compact/queue minimum and default sizes in both appearances.
- [x] Complete ordered checks and two review cycles, then deliver a feature PR to dev.

## Implementation and established patterns

The existing App and audio hook remain mounted while the full workspace is hidden.
The compact view composes the existing transport and cached artwork reader. Queue
snapshots and entry identity remain owned by the current audio hook, so duplicate
tracks and auto-advance use the same queue as the full player.

Native size changes follow Electron's BrowserWindow setContentSize, setMinimumSize,
getNormalBounds and setBounds APIs, preserving the single renderer/audio owner:
https://www.electronjs.org/docs/latest/api/browser-window
The narrow context-isolated preload method follows Electron's IPC sender-validation
pattern. No third-party implementation code or reference artwork is copied.

This is a compact presentation inside the same native window. It retains native
window chrome and does not claim cross-platform OS translucency. Waveform analysis
and its cache, shuffle/repeat modes and always-on-top are separate future slices;
no decorative waveform or inactive controls are shown.

Native test captures use fictional tracks and original geometric SVG cover art
exported to PNG. A disposable profile and isolated Xvfb display avoid the user's
catalogue, credentials and currently running player. Capture with
MUZO_COMPACT_CAPTURE_DIR=/tmp/muzo-compact-player-captures yarn desktop test electron/compact-player-layout.test.ts.

All ordered gates pass with 348 tests across 83 files. Initial Standards review
found one P3 naming issue; a behavior-preserving refactor names native dimensions.
Both final review axes have zero findings. See the
[review record](../../reviews/2026-10-24-compact-player.md).
