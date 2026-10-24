# Card 040 review record

Base: `e805aae3cdf57f55468d2b1189d71f7514faa8f2`.
Initial reviewed head: `1dc5a51ef4850f2fb83bd525ce968213a600d9aa`.
Final reviewed head: `fd7c1455287a0a4d75606b19d80d3f232643817f`.
Both cycles used the complete three-dot diff and full atomic commit list.

## Initial Standards

Hard documented violation:

- [P3] Name the native sizing policy in apps/desktop/electron/player-window-mode.ts:51-52. The calls embed `640`, `360`, `480`, `740`, `420` and `560` directly. AGENTS.md, Things this repo does NOT do, requires: "No magic numbers - name them as value objects or constants." Extract named compact/queue sizing constants or a small mode-sizing table, preserving the existing behavior and focused sizing tests in a separate refactoring commit.

Judgement-call smells: none actionable.

Reviewed the pinned diff from e805aae3cdf57f55468d2b1189d71f7514faa8f2 to 1dc5a51ef4850f2fb83bd525ce968213a600d9aa in the compact-player worktree, including the App change without whitespace noise. One audio/state owner remains mounted; compact presentation composes existing transport/artwork, and queue entry identity stays with the audio hook. Native IPC validation, typed Result handling, rollback and focus restoration have focused modules and meaningful coverage. The history separates red regressions, implementation and refactoring.

Card 040 records the established Electron/single-renderer pattern. No speculative waveform, shuffle/repeat or always-on-top machinery is introduced. Tool-enforced concerns are excluded. The supplied gate record confirms 348 tests and ordered install/format/lint/types/build/package/make success, plus eight native geometry scenarios.

Findings: 1 hard violation; 0 judgement-call smells.


## Initial Spec

# Spec review: compact player, initial cycle

Pinned diff: `e805aae3cdf57f55468d2b1189d71f7514faa8f2...1dc5a51ef4850f2fb83bd525ce968213a600d9aa` in the dedicated compact-player worktree. Confirmed both refs, the nonempty full diff and all 24 focused commits against card 040.

## (a) Missing or partial requirements

No actionable findings in the pinned implementation.

## (b) Unasked scope

No actionable findings. Waveform analysis, shuffle/repeat, always-on-top and OS translucency remain outside this slice; no decorative waveform or inactive feature controls appear.

## (c) Apparently implemented requirements done incorrectly

No actionable findings. The App/audio owner stays mounted while the library workspace is hidden, with compact controls bound to the same playback state. Source/playlist state is retained, satisfying "Enter compact mode and return to the existing library view without reloading audio, resetting position or losing the selected playlist/source." Integration coverage checks one audio instance, one source preparation, retained position and restored view/focus.

The compact view shows real artwork, metadata, progress, transport and volume, with readable playback errors. Its optional queue uses the current audio hook's entries and entry IDs; the duplicate-track regression verifies selection without replacing the queue. Independently inspected the native dark queue capture, which shows the live queue and artwork-led player with the saved material.

Native transitions validate modes and main-frame/window ownership, return typed failures and preserve snapshots for bounds/minimum/maximized restoration or rollback. Renderer state changes only after success. Focus targets move into the visible player and return to the library control. Native frame configuration remains unchanged. Electron's [BrowserWindow API](https://www.electronjs.org/docs/latest/api/browser-window) supports the selected sizing/restoration operations.

Native layout coverage exercises real IPC and compact/queue default/minimum geometry in both appearances. The parent reports all ordered gates passed with 348 tests across 83 files. This read-only review did not duplicate those gates, access user data or modify repository/browser state.


## Final Standards

Hard documented violations: none remaining.

Judgement-call smells: none actionable in the updated pinned Card 040 diff from e805aae3cdf57f55468d2b1189d71f7514faa8f2 to fd7c1455287a0a4d75606b19d80d3f232643817f, reviewed in the compact-player worktree.

The initial P3 sizing finding is resolved. COMPACT_DIMENSIONS, COMPACT_MINIMUM_HEIGHT and COMPACT_HEIGHT now name the native policy, replacing the embedded dimensions and repeated mode ternaries. The focused behavior-preserving refactoring retains the exact tested values without introducing speculative configuration machinery.

Rechecked native boundary and UI coordination: sender-frame validation and a narrow preload method return typed Results; rollback retains window bounds/minimum/maximized state; one mounted App/audio owner provides transport and queue identity to both views. Presentation, queue rendering, window state and native sizing remain cohesive modules. The commit history preserves separate red regressions, implementations and refactoring.

The twelve-smell baseline raises no further actionable findings; tool-enforced concerns are excluded. The parent reports all 348 tests and ordered gates passed before the refactoring, then controller and real compact-geometry checks passed again afterward. No production changes beyond naming the sizing policy were added for this review.

Findings: 0 hard violations; 0 judgement-call smells.


## Final Spec

# Spec review: compact player, second cycle

Pinned diff: `e805aae3cdf57f55468d2b1189d71f7514faa8f2...fd7c1455287a0a4d75606b19d80d3f232643817f` in the dedicated compact-player worktree. Rechecked the full changed-file set, commit sequence and focused dimension-naming refactor against card 040. The initial Spec review required no correction.

## (a) Missing or partial requirements

No actionable findings.

## (b) Unasked scope

No actionable findings. Named dimensions clarify the existing native policy without changing window sizes or adding features. Deferred waveform analysis, shuffle/repeat and always-on-top remain absent.

## (c) Apparently implemented requirements done incorrectly

No actionable findings on fresh inspection. Compact and expanded-queue dimensions retain their prior values. Snapshot-based restoration still preserves original bounds, minimum size and maximized state, with rejected operations rolling back through typed Results, satisfying "Restore the original native window bounds, minimum size and maximized state; roll back rejected resize operations."

The complete slice retains one mounted App/audio owner, hidden library workspace and visible compact controls bound to real progress/volume/playback state. Live queue entries retain duplicate-entry identity, source/playlist selection survives restoration, and keyboard focus moves between visible modes. Artwork/metadata/error presentation and the saved material/accent remain intact. Native OS chrome is preserved.

The parent reports all 348 tests across 83 files and ordered gates passed before the behavior-preserving refactor, then controller units and real compact geometry passed again afterward. Native coverage continues to exercise trusted IPC and default/minimum compact/queue sizes in both appearances. This read-only re-review did not duplicate those runs, change repository/browser state or access user catalogue data.


## Verification

All ordered gates were repeated after the focused dimension refactor: Yarn
install, format check, lint, strict types, 348 tests in 83 files, desktop build,
Linux Forge package and ZIP/DEB make pass. The existing audio element, source,
position and source/playlist state persist between full and compact views.
Native geometry tests exercise the actual App, context-isolated preload and
Electron mouse input across eight light/dark/default/minimum scenarios.

The real native screenshots use a disposable profile, fictional metadata and
original cover designs. Root visually inspected the 740px queue view. No user
catalogue, credentials or currently running player were touched. Waveform
analysis/caching and always-on-top remain outside this first compact slice.

Totals: initial Standards 1 P3, Spec 0; final Standards 0, Spec 0. The naming
finding was fixed in a separate refactor commit and native tests stayed green.
