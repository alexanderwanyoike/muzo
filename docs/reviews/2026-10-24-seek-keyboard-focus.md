# Card 038 review record

Base: `4e23fd61c3efa927ee7d6c0bded62d3e643667e2` (PR #63).
Reviewed head: `6e602001902033b7c5ed998ed8ea170fa72dace7`.
Both cycles reviewed the full three-dot diff and three atomic commits.

## Initial Standards

Hard documented violations: none found.

Judgement-call smells: none actionable in the pinned Card 038 diff from 4e23fd61c3efa927ee7d6c0bded62d3e643667e2 to 6e602001902033b7c5ed998ed8ea170fa72dace7.

The sole production change narrows the visible seek-wrapper focus selector to the browser's focus-visible state. It preserves the native range input and keyboard behavior without adding state, event handlers or an abstraction. The fixture renders the actual NowPlayingBar and production CSS, uses real Electron pointer/keyboard input, checks light/dark appearances and verifies arrow-key adjustment as well as visible focus.

Renderer setup, native interaction and test orchestration have separate cohesive files. The test uses an isolated disposable profile, bounded execution and finally cleanup. The history keeps the observed failing native regression separate from the minimal selector fix, with CI/card documentation in a subsequent focused commit. Card 038 records the established input-modality and existing focus-proxy pattern.

The twelve-smell baseline raises no actionable findings; tool-enforced concerns are excluded. Ordered repository gates and final re-review remain with the parent. This review performs no repository merge or production edit.

Findings: 0 hard violations; 0 judgement-call smells.


## Initial Spec

# Spec review: seek keyboard focus, initial cycle

Pinned diff: `4e23fd61c3efa927ee7d6c0bded62d3e643667e2...6e602001902033b7c5ed998ed8ea170fa72dace7`. Reviewed all three commits and the complete diff against card 038 and the reported pointer-focus rectangle.

## (a) Missing or partial requirements

No actionable findings. Refreshing README/site captures is explicitly subsequent work rather than part of this focused fix.

## (b) Unasked scope

No actionable findings. The native fixture and platform-CI entry directly verify the requested pointer/keyboard distinction. No transport handler, playback or screenshot asset change is introduced.

## (c) Apparently implemented requirements done incorrectly

No actionable findings. The only production change replaces wrapper `:focus-within` with a descendant range `:focus-visible` condition. The range retains focus and its existing seeking behavior, while the visible shell projects the keyboard focus indicator. This meets "Mouse seeking leaves no blue focus rectangle around the timeline" without discarding "Keyboard navigation still shows a clear focus indicator and arrow keys seek."

The native fixture renders the actual NowPlayingBar and production CSS in a disposable profile. It sends pointer input, reverse/forward Tab and an arrow key in both appearances. Assertions require a focused range without a pointer outline, a solid 2px keyboard outline and increased seek value. This directly covers the reproduction requirement rather than merely asserting a selector string.

The commit sequence separates the failing native regression from the minimal implementation. Linux runs the normal suite under an isolated Xvfb display; Windows/macOS native CI now includes the same regression. Fictional metadata and the stub artwork boundary avoid user catalogue or Dropbox access.

The parent reports the intended native red/green result. Final ordered gates and screenshot refresh remain pending; this read-only review does not certify those tasks complete and changed no repository/browser state.


## Final Standards

Hard documented violations: none found.

Judgement-call smells: none actionable on the second review of the pinned Card 038 diff from 4e23fd61c3efa927ee7d6c0bded62d3e643667e2 to 6e602001902033b7c5ed998ed8ea170fa72dace7.

The sole production selector change keeps browser input-modality ownership intact: pointer seeking receives no wrapper outline, while keyboard focus still receives the existing indicator. The native regression uses the real transport/CSS and actual pointer/Tab/arrow input in light/dark modes. Test setup, native interaction and orchestration remain cohesive, with isolated profiles, bounded execution and finally cleanup.

The observed failing native regression and minimal fix remain separate commits, followed by CI/card integration. Card 038 records the established focus-visible pattern and existing application focus proxy. No unnecessary state, framework dependency or error-handling path is added. The twelve-smell baseline raises no actionable findings; tool-enforced concerns are excluded.

The parent reports install/format/lint/types and all 338 tests passing. Actual-App browser reproduction also confirmed pointer focus without the outline. Build, package and make are ongoing and are not claimed complete here.

Findings: 0 hard violations; 0 judgement-call smells.


## Final Spec

# Spec review: seek keyboard focus, second cycle

Pinned diff: `4e23fd61c3efa927ee7d6c0bded62d3e643667e2...6e602001902033b7c5ed998ed8ea170fa72dace7`. Rechecked the full file set, three commits and production selector. No initial finding required a fix.

## (a) Missing or partial requirements

No actionable findings. Screenshot replacement and site work remain separate follow-up slices as specified.

## (b) Unasked scope

No actionable findings. The production diff remains one focused CSS selector change; supporting additions exercise native focus behavior and extend the existing platform-CI command.

## (c) Apparently implemented requirements done incorrectly

No actionable findings. The wrapper outline depends on the seek range matching `:focus-visible`, preserving the keyboard proxy while avoiding the previous unconditional focus-within rectangle. The range input and its existing onChange seeking handler are unchanged.

The native regression still requires focused pointer input with no visible outline, then keyboard focus with a solid 2px outline and increased value after an arrow key, in both appearances. This satisfies "Mouse seeking leaves no blue focus rectangle around the timeline" and "Keyboard navigation still shows a clear focus indicator and arrow keys seek."

The parent additionally reports actual-App browser input produced focused=true, focus-visible=false and no wrapper outline. Independently inspected `/tmp/muzo-player-fixed.jpg`; the transport timeline is visibly clean in the fresh capture. This supports the reported user-facing fix without replacing the keyboard assertions.

The parent reports install, formatting, lint, type checks and all 338 tests across 78 files passed. Build, package and make remain ongoing and are not certified complete here. This read-only review changed no repository/browser state and did not duplicate gate runs.


## Verification

The native regression was committed red at b934216, failing because pointer
focus drew a solid outline. The CSS-only fix at 92fe5e2 makes it pass in light
and dark, with a 2px keyboard outline and working arrow-key seeking. The test
uses actual mouse/Tab/arrow input against the real React transport and styles.
Native fixture frames are awaited before measuring rendered state.

Yarn install, format check, lint, strict types, all 338 tests (78 files), desktop
build, package and Linux ZIP/DEB make pass. Windows/macOS CI now includes the
native regression; those remote results are separate from the local Linux checks.

The original full listening fixture was clicked at the same seek position after
the fix: focused=true, focus-visible=false, wrapper outline=none. Captured a fresh
1440 x 900 screenshot for the subsequent README update. No debug logs or production
JavaScript were added. The existing user app and catalogue were not touched.

Totals: Standards 0, Spec 0 in each cycle; no fixes required after review.
