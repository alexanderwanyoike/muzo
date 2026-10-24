- Sprint: 02
- Type: Desktop appearance
- Status: In progress

# Card 046 - Reveal the desktop through Frosted glass

The user explicitly chose seeing the desktop through the window. The existing
Glass material only composites app artwork inside an opaque OS window.

## Acceptance criteria

- [ ] Selecting Frosted glass reveals the desktop through the library and mini player.
- [ ] Other materials remain opaque, including after switching back or restarting.
- [ ] Preserve readable text, opaque artwork/controls, rounded edges and playback.
- [ ] Respect reduced transparency and retain an opaque fallback on unavailable systems.
- [ ] Use native macOS vibrancy and Windows 11 acrylic where available; Linux uses compositor alpha.
- [ ] Verify native compositing with a changing backdrop, resizing and theme switching.
- [ ] Run ordered gates, two review cycles and deliver a feature PR to dev.

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
