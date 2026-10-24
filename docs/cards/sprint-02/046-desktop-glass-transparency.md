- Sprint: 02
- Type: Desktop appearance
- Status: In review

# Card 046 - Reveal the desktop through Frosted glass

The user explicitly chose seeing the desktop through the window. The existing
Glass material only composites app artwork inside an opaque OS window.

## Acceptance criteria

- [x] Selecting Frosted glass reveals the desktop through the library and mini player.
- [x] Other materials remain opaque, including after switching back or restarting.
- [x] Preserve readable text, opaque artwork/controls, rounded edges and playback.
- [x] Respect reduced transparency and retain an opaque fallback on unavailable systems.
- [x] Use native macOS vibrancy and Windows 11 acrylic where available; Linux uses compositor alpha.
- [x] Verify native compositing with a changing backdrop, resizing and theme switching.
- [x] Run ordered gates, two review cycles and deliver a feature PR to dev.

## Established pattern and platform scope

Follow Electron 43's official transparent-window and native-background APIs:
https://github.com/electron/electron/blob/v43.0.0/docs/tutorial/custom-window-styles.md
https://github.com/electron/electron/blob/v43.0.0/docs/api/structures/base-window-options.md
https://github.com/electron/electron/blob/v43.0.0/docs/api/browser-window.md

Linux keeps its native frame and enables the transparent native surface at
creation. The composited X11 native probe confirms alpha and resizing work with
Electron 43 and Metacity; Cinnamon's compositor uses the same alpha surface.
CSS cannot blur other desktop applications. Native desktop blur is provided by
macOS vibrancy and Windows 11 22H2 acrylic; Linux blur depends on its compositor.
Windows versions without the native material API retain the opaque fallback.
Do not fade the whole window, including text and artwork, with window opacity.

Depends on #67/#68/#73/#72 (compact player, satin aluminium, Glass and corners)
and #70 (MIT). It is branched from dev with those reviewed prerequisites.

Windows uses Electron's native window-controls overlay because its acrylic
client backdrop requires a hidden native frame in Electron 43. Native window
buttons remain, playback controls reserve their area and the header is draggable.
Windows acrylic windows keep compositor-owned rounded corners to preserve DWM
backdrops. macOS and Linux retain their existing native frames.
The real compositing test requires Metacity in an isolated Xvfb session on CI.

## Verification

All ordered checks pass on source head 31e3d9d: frozen Yarn install, formatting,
lint, strict types, 365 tests in 86 files, build/package and Linux ZIP/DEB make.
Both initial and final reviews returned zero findings on Standards and Spec.
The native regression first failed at alpha 255, then passes with translucent
surfaces. A separate compositing assertion rejected an occluding test backdrop
and now matches the expected desktop/player colour within six channel values.
Native captures and conservative black/white contrast bounds verify readable
light/dark library, mini and resized surfaces. Reduced transparency and other
materials restore alpha 255; library bounds restore after mini mode.
See [the review record](../../reviews/2026-10-24-desktop-glass-transparency.md).
