- Sprint: 02
- Type: Bug fix
- Status: In review

# Card 038 - Keep the seek bar clean after pointer input

The user noticed a blue rectangular focus border around the playback seek bar
in the new screenshot and requested a fix before refreshing the captures.

## Acceptance criteria

- [x] Mouse seeking leaves no blue focus rectangle around the timeline.
- [x] Keyboard navigation still shows a clear focus indicator and arrow keys seek.
- [x] Reproduce the issue with native input against the actual transport in light and dark appearances.
- [x] Commit the red regression test separately from the minimal fix.
- [x] Include the native regression in Linux, Windows and macOS verification.
- [x] Complete ordered repository gates and both review cycles; hand off through a feature PR to dev.

The visible wrapper used :focus-within around an invisible range input, which
made every pointer focus draw an outline. Use :has(input:focus-visible) to project
the browser's keyboard-focus decision onto the visible wrapper. This follows
MDN's established input-modality pattern and the app's existing appearance-radio
focus proxy, without copying third-party implementation code:
https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Selectors/:focus-visible

The native test reproduces the original solid outline after a mouse click, then
checks a clean pointer state, a 2px keyboard outline and arrow-key value changes.
It uses a disposable profile, fictional metadata and the repo's existing Electron
fixture pattern. A visible fixture window permits deterministic native frames;
Linux runs it inside its own Xvfb display. No user library or Dropbox access is used.

Depends on pending merge-policy PR #63. README screenshots and the Pages site are
separate work and will be refreshed to include this fix before their handoff.

All ordered local gates pass with 338 tests, including native pointer/keyboard
coverage. Both review cycles have zero findings on both axes. See the
[review record](../../reviews/2026-10-24-seek-keyboard-focus.md).
