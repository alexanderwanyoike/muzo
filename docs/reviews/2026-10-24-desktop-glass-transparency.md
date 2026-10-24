# Card 046 review record

Base: `dec53ef6bac943d01257f7d9150a5879d35fd341` (reviewed prerequisite stack).
Initial reviewed head: `010cd804fba1d3e409d59525d74f1f3128a51eee`.
Final reviewed head: `31e3d9d22667545ac28590521067b5e4b24d26c7`.
Both cycles used the complete three-dot diff and commit list.

## Initial Standards

# Native desktop glass initial Standards review

Base `dec53ef6bac943d01257f7d9150a5879d35fd341`; reviewed head `010cd804fba1d3e409d59525d74f1f3128a51eee`.
Reviewed the complete three-dot diff and twenty-one focused commits against AGENTS.md, Card 046 and the twelve Fowler smell heuristics. Viewed both supplied real native screenshots.

Hard documented violations: none found.

Judgement-call smells: none actionable.

The change keeps framework-specific window effects at the Electron edge. A cohesive native-effect function, validated IPC installer, renderer hook and shared window factory have distinct responsibilities. Expected unsupported-platform and native API failures use typed Result values; the renderer applies its desktop-glass flag only after a successful enabled result and retains a compatible opaque path for older bridges. Main-frame and boolean validation follow the existing narrow IPC boundary.

Platform selection remains explicit and small. The named Windows build threshold gates acrylic, while native window-controls overlay and compositor-owned corners preserve its backdrop. macOS vibrancy and Linux compositor alpha use established Electron APIs documented in the card. The work does not fade text/artwork with whole-window opacity or introduce a custom rendering service. Reduced-transparency behavior and conservative surface tint preserve the existing semantic theme tokens.

Atomic red/green history covers appearance synchronization, native effects, unsupported Windows, reduced transparency, the isolated bridge, actual pixel compositing and native overlay/corner behavior. Extracting the shared production window factory remains a separate refactoring commit. Native verification now checks the real production factory against a changing backdrop, with a pixel-compositing assertion guarding the previously misplaced backdrop. The fixture isolates its Metacity process under Xvfb and cleans up native resources.

The native screenshots show readable library/mini surfaces, opaque cover art and controls, native frames and rounded mini edges. CI adds the necessary isolated compositor and native-platform tests. Parent-owned ordered gates and PR delivery are still in progress and are not certified by this read-only review.

Findings: 0 hard violations; 0 judgement-call smells.


## Initial Spec

# Spec review: native desktop glass, initial cycle

Pinned diff: `dec53ef6bac943d01257f7d9150a5879d35fd341...010cd804fba1d3e409d59525d74f1f3128a51eee`. Reviewed the complete 21-file diff and 21 commits against Card 046, including the actual native desktop captures.

## Missing or partial requirements

No actionable findings. The production window factory, validated preload command and appearance hook enable a native transparent surface only for Frosted glass (Card 046 lines 12-13). Renderer surfaces retain partial alpha without fading artwork or text via window opacity. Theme restoration removes the native glass attribute and requests an opaque background.

## Unasked scope

No actionable findings. The production window factory is shared with native verification, and the Windows native controls overlay supports acrylic without replacing system buttons. The small desktop-compositing fixture and CI compositor dependency directly support line 17's verification requirement.

## Apparently implemented requirements done incorrectly

No actionable findings. macOS uses under-window vibrancy; supported Windows 11 builds use acrylic; Linux retains its frame and compositor alpha. Older Windows reports the unavailable material and keeps renderer surfaces opaque. Reduced transparency disables native glass at the boundary, while the media-query fallback restores an opaque root (lines 15-16).

Windows overlay controls reserve their native area and remain excluded from draggable transport/header regions. Acrylic windows retain compositor-owned corners; Linux retains the prior native corner handling, bounds restoration and shared playback session (line 14).

The composited native screenshots show the actual player above the desktop, including the rounded mini window. The fixture changes the desktop backdrop and verifies both pixel change and the expected foreground-over-backdrop composite, preventing the earlier backdrop-above-player false positive. It also checks light/dark library, mini and resized alpha, conservative text contrast, opaque theme/reduced-transparency restoration, bounds and native control collisions. Linux desktop blur limitations are explicitly documented instead of being presented as compositor-independent blur.

Findings: 0. This read-only review did not run suites or certify unfinished gates, Windows/macOS CI, or PR handoff at line 18.


## Final Standards

# Native desktop glass final Standards review

Base `dec53ef6bac943d01257f7d9150a5879d35fd341`; reviewed head `31e3d9d22667545ac28590521067b5e4b24d26c7`.
Rechecked the full effective diff and commit list, including the single correction after the initial `010cd804fba1d3e409d59525d74f1f3128a51eee` review, against AGENTS.md and the twelve Fowler smell heuristics.

Hard documented violations: none found.

Judgement-call smells: none actionable.

The correction removes an unused fixture import and classifies the extracted Electron window-construction entry point alongside existing main/preload runtime imports. It does not weaken unsafe-value, explicit-any or strict-TypeScript rules, and production behavior remains unchanged from the initial review.

The native effect, validated main-frame IPC boundary, renderer synchronization hook and window factory retain cohesive responsibilities. Unsupported Windows and native-effect failures return typed Results; opacity is exposed in CSS only after native success. macOS vibrancy, supported Windows acrylic and Linux compositor alpha remain at the framework edge, with opaque and reduced-transparency fallbacks. Native Windows controls and compositor corners preserve the acrylic path without a custom titlebar implementation.

Rechecked the atomic red/green and separate refactor history, native pixel-compositing regression, real production factory use, overlay collision assertions and isolated compositor cleanup. Previously inspected native images continue to support the readable, composited library/mini presentation with opaque cover art and native controls. No additional source alteration invalidates those captures.

The parent reports the complete serialized ordered gates passed: 365 tests in 86 files, formatting, lint, strict types, desktop build/package and Linux ZIP/DEB make, using two workers to limit resource pressure. This read-only re-review did not rerun those suites or perform a merge/publication. Card completion and PR handoff remain parent-owned final documentation work.

Findings: 0 hard violations; 0 judgement-call smells.


## Final Spec

# Spec review: native desktop glass, final cycle

Pinned diff: `dec53ef6bac943d01257f7d9150a5879d35fd341...31e3d9d22667545ac28590521067b5e4b24d26c7`. Rechecked the complete changed-file set and the delta from the initial zero-finding pin `010cd80`. The only subsequent changes classify the extracted Electron constructor entry point for dependency linting and remove an unused fixture import; runtime behavior is unchanged.

## Missing or partial requirements

No actionable findings. Frosted glass coordinates the native surface with renderer alpha in library and mini views, while other materials restore opacity (Card 046 lines 12-13). Native macOS vibrancy, supported Windows acrylic and Linux compositor alpha follow the documented platform scope.

## Unasked scope

No actionable findings. The shared production window factory and Windows native controls overlay directly support native glass and its real-window verification. Existing native buttons, safe draggable header areas, shared playback and bounds restoration remain intact.

## Apparently implemented requirements done incorrectly

No actionable findings. System reduced transparency and the renderer media fallback retain an opaque surface; unsupported Windows avoids its native material API and exposes the fallback deliberately (lines 15-16). Windows acrylic retains compositor-owned corners, while Linux keeps its prior rounded client region.

The previously inspected unedited native screenshots show the actual player above changing desktop colours, with legible text and opaque artwork. Pixel verification checks the foreground-over-backdrop result as well as pixel change, preventing a hidden-player/backdrop-only false positive. Light/dark library, mini and resized samples cover partial alpha and contrast, with opaque reduced-preference/theme restoration, original bounds and caption-control collision checks (lines 14, 17).

The parent reports all ordered gates green: 365 tests in 86 files, native compositing/control/contrast checks, build, package and Linux ZIP/DEB make. This read-only re-review did not repeat those suites or certify pending Windows/macOS remote CI and PR delivery. Card completion remains a parent handoff step.

Findings: 0.


## Verification and limits

All ordered local gates passed with 365 tests in 86 files and Linux ZIP/DEB
packaging. Red and green commits are separate for renderer synchronization,
native surfaces, older-Windows fallback, reduced transparency, isolated IPC,
real native compositing and acrylic-native caption/corner behavior.

The native integration uses the real app, preload, production window factory,
window commands and a disposable composited X11 desktop. It verifies six actual
light/dark library/mini/resized surface samples, text contrast at least 4.5:1 over
both black and white backdrops, a changed desktop showing through the player,
correct composite pixels, opaque Studio/reduced-transparency fallback, restored
window size and native title-bar-overlay hitboxes. The first backdrop fixture
occluded the player; a new assertion failed on it, then the fixture was fixed.
The real native-alpha regression had already failed at 255 before implementation.

Linux uses native compositor alpha with a native frame; desktop blur is compositor
provided. macOS uses under-window vibrancy. Windows 11 22H2+ uses native acrylic
with the native controls overlay and compositor-owned rounded corners. Older
Windows reports a typed unavailability and retains the opaque material. No window
opacity is applied to text, artwork or controls. Electron's native platform
APIs and their limits are recorded in Card 046. Cross-platform CI runs the native
factory and overlay checks on Windows/macOS after feature PR creation.
