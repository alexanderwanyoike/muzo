- Sprint: 02
- Type: Feature
- Status: Done
- Depends on: 023 (PR #50)

# Card 024 - Remove unused desktop menus

Remove the default File/Edit/View/Window menu strip. Muzo's own controls provide
navigation and playback. Windows and Linux should have no application menu, even
when Alt is pressed. macOS keeps its native Muzo app controls for Hide and Quit,
with hidden native editing/close actions retaining standard keyboard shortcuts.

## Acceptance criteria

- [x] Suppress the default menu rather than temporarily hiding its strip.
- [x] Install the policy before the first window and preserve it for later windows.
- [x] Keep standard macOS Quit/Hide, undo/redo, cut/copy/paste, select-all and close shortcuts.
- [x] Keep the native window frame and all renderer controls.
- [x] Complete ordered repository checks and two review cycles (275 tests).

## Established pattern

Use Electron's MIT-licensed Menu API: setApplicationMenu(null) removes the Windows
and Linux menu strip and prevents the default menu from being created.
https://www.electronjs.org/docs/latest/api/menu#menusetapplicationmenumenu
On macOS, native role actions with visible:false and
acceleratorWorksWhenHidden:true preserve editing keys without those menu titles:
https://www.electronjs.org/docs/latest/api/menu-item
No external code is copied and no dependency is needed.

## Verification

Platform-policy tests cover Linux/Windows removal and the macOS menu/shortcut
contract. Inspect the rendered Linux window after restarting the player. Native
macOS behavior requires an actual macOS session; platform-policy tests run here.

Review evidence: [two review cycles](../../reviews/2026-10-24-minimal-desktop-menu.md).
