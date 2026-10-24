# Card 044 review record

Base: `50870946527d6058c8279660c4c8c45179f9be1e`.
Both cycles reviewed implementation head
`b83303c533baa84c1f27e099559332da050ba753`, the full diff and atomic commits.

## Initial Standards

Hard documented violations: none found.

Judgement-call smells: none actionable in the updated pinned Card 044 diff from 50870946527d6058c8279660c4c8c45179f9be1e to b83303c533baa84c1f27e099559332da050ba753, reviewed in the rounded-mini-player worktree.

Native shape geometry is isolated in a small pure helper with a named radius; window-mode ownership retains sizing, rollback and lifecycle coordination. Expected shaping failures use Results, asynchronous resize failures attempt rectangular recovery and report failed recovery through the logger. Resize listeners are removed on close. Platform selection preserves compositor handling on macOS/Wayland and documents the observed X11 empty-region discrepancy.

The implementation retains the existing native frame, renderer/audio owner and narrow bridge rather than adding transparent/secondary windows or speculative title-bar machinery. The focused red/green history covers corners, resizing, native title-bar seams, rollback and the explicit full-rectangle X11 restoration. The final import-order commit is behavior-preserving.

Card 044 records established Electron APIs and native constraints. Actual XShape region evidence and the inspected native queue capture support bottom-corner clipping with an intact title-bar join. Existing native geometry/input coverage remains maintained. The twelve-smell baseline raises no actionable findings; tool-enforced concerns are excluded. Final gate completion, second review and feature-PR handoff remain with the parent.

Findings: 0 hard violations; 0 judgement-call smells.

## Initial Spec

# Card044 initial Spec review

Pinned diff: `50870946527d6058c8279660c4c8c45179f9be1e...b83303c533baa84c1f27e099559332da050ba753`. Reviewed the full diff and thirteen commits against Card044, the supplied native captures and XShape records.

## (a) Missing or partial requirements

No actionable findings. The existing window/controller remains responsible for compact geometry and restore. Audio/session ownership and renderer state are untouched. Windows uses rounded native regions; X11 rounds bottom client corners while preserving the title-bar seam. macOS/native Wayland intentionally keep compositor treatment, as required by "macOS and native Wayland retain compositor-provided corner treatment."

Ordered gates and handoff remain intentionally pending. The supplied actual native evidence covers X11; this review does not certify a Windows/macOS native visual run.

## (b) Unasked scope

No actionable findings. The patch introduces no second window, custom chrome, new state bridge, transparent-window architecture or playback feature. Changes stay within region generation, existing window-mode integration, matching renderer clipping and verification/documentation.

## (c) Implemented incorrectly

No actionable findings. Native captures and region queries support "X11 compact modes round the native client bottom corners, preserving the straight join to the separately drawn native title bar and its own top corners." Latest compact regions exclude both bottom corners, retain their centre, and follow the four supplied solo/queue sizes.

The restored 1200x800 native region is one complete rectangle. Linux's explicit rectangle and subsequent resize synchronization address the observed empty-region reset behavior, meeting "Library mode has a full rectangular drawing/input region and remains resizable." Existing bounds/minimum/maximized restoration logic remains in place.

Shape failures return typed Results, transition failures restore prior geometry/shape, and resize failures attempt a rectangular recovery, matching "Native errors use Result rollback; resize errors try a rectangular fallback." Close removes the resize listener. Renderer bottom clipping matches the named native radius across materials.

This review did not alter repository/external state or duplicate the parent's verification runs.

## Final Standards

Hard documented violations: none found.

Judgement-call smells: none actionable on the second review of Card 044 from 50870946527d6058c8279660c4c8c45179f9be1e to b83303c533baa84c1f27e099559332da050ba753, reviewed in the rounded-mini-player worktree.

Rechecked the small shape helper, native mode/lifecycle coordination and renderer edge treatment. Expected shape failures remain typed Results with rollback; resize failures attempt a rectangular fallback and report failed recovery. Listener cleanup and unsupported-platform behavior remain explicit. Native frame, audio ownership and the existing bridge stay intact.

The X11 full-rectangle reset is constrained to the documented Electron discrepancy and follows later library resizes. Windows retains its documented reset path, while macOS/Wayland retain compositor behavior. Separate meaningful red/green commits cover shape geometry, title-bar seam preservation, rollback and restored library content; subsequent capture/import-order changes are focused verification/formatting.

The twelve-smell baseline raises no actionable findings; tool-enforced concerns are excluded. The parent reports 356 tests and native sixteen-scenario geometry/input checks passing. Previously inspected native capture and XShape regions show excluded bottom corners, intact title-bar join and restored full library region. No implementation corrections were required after initial review.

Findings: 0 hard violations; 0 judgement-call smells.

## Final Spec

# Card044 final Spec review

Pinned diff: `50870946527d6058c8279660c4c8c45179f9be1e...b83303c533baa84c1f27e099559332da050ba753`. Freshly reviewed the complete diff, thirteen commits and Card044. The initial review required no fixes; this head is unchanged.

## (a) Missing or partial requirements

No actionable findings. The existing BrowserWindow, native controls, playback/session ownership and library snapshot restoration remain intact. Shape generation follows compact mode and resize; renderer clipping follows its bottom radius. Compositor fallback is explicit for macOS/native Wayland, matching "macOS and native Wayland retain compositor-provided corner treatment."

The parent reports 356 tests and all 16 native XShape scenarios passed. Supplied native captures/regions support X11 behavior; this review does not claim a Windows/macOS visual run. PR handoff remains pending.

## (b) Unasked scope

No actionable findings. Changes remain limited to native regions, existing controller recovery, renderer clipping, tests and documentation. No transparent/second window, frameless controls or playback feature is added.

## (c) Implemented incorrectly

No actionable findings. X11 preserves the native title-bar seam and excludes both compact bottom corners, satisfying "X11 compact modes round the native client bottom corners, preserving the straight join to the separately drawn native title bar and its own top corners."

The supplied restored region is a full 1200x800 rectangle. Linux's explicit rectangle and subsequent resize synchronization meet "Library mode has a full rectangular drawing/input region and remains resizable." Bounds, minimum size and maximized-state restoration remain preserved.

Typed transition failures roll back geometry/shape, resize failures attempt rectangular recovery, and closing removes the resize listener. Existing queue, keyboard, contrast and restoration assertions remain present. This review was read-only and did not rerun the parent's gates.

## Verification

All ordered gates passed on the reviewed implementation head: Yarn install,
formatting, lint, strict types, 356 tests in 83 files, desktop build, Linux x64
package and ZIP/DEB make. The existing native Settings matrix covers 56 scenarios;
compact verification covers 16 aluminium/glass light/dark default/minimum cases
with the real preload, keyboard/wheel input, shape extents and contrast checks.

Independent native verification used a disposable Xvfb display with Metacity and
queried actual XShape rectangles. All 16 compact cases excluded both bottom
corners, included centre points and preserved the title-bar seam. Library restore
produced one full 1200x800 rectangle. Full root desktop captures retained actual
native clipping and decorations, with original fictional artwork and no user data.

The Linux full-rectangle reset addresses the reproduced Electron 43/X11 empty
region from `setShape([])` and follows later library resizes. Windows uses the
documented reset; macOS/native Wayland retain compositor treatment. Native
Windows/macOS visual verification is not claimed by this local Linux run.

Totals: Standards 0, Spec 0 in both cycles; no fixes required after review.
