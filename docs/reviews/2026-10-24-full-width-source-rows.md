# Card 045 review record

Base: `f3897c1` (dev). Reviewed head: `9c0e4a6`.
Full three-dot diff and commit list reviewed twice.

## Initial Standards

Hard documented violations: none found.

Judgement-call smells: none actionable in the pinned Card 045 diff from f3897c1f56f872cebee6d3aec857baae7db53e45 to 9c0e4a67a0fa8f50fa0cc74ead9deb3cc90b2e28, reviewed in the full-width-source-rows worktree.

The sole production change removes the unused reserved scrollbar gutter from the existing shared list rule. It preserves native overflow behavior without introducing layout state, a scrolling engine or unrelated UI scaffolding. Sources and Playlists receive the correction through their existing shared styling responsibility.

The native regression extends the maintained Settings measurement/assertion boundary rather than duplicating a test harness. It measures actual row/panel edges and overflow across existing native scenarios. The observed failing geometry test is committed separately before the minimal CSS fix; documentation and formatting remain separate commits. Card 045 records the established native CSS overflow pattern and unchanged interaction scope.

Application error handling, architecture and domain boundaries are unchanged. The user's latest merge-commit instruction takes precedence over the older checkout's squash wording. The twelve-smell baseline raises no actionable findings; tool-enforced concerns are excluded. Final ordered checks, second review and feature-PR delivery remain with the parent.

Findings: 0 hard violations; 0 judgement-call smells.


## Initial Spec

# Card045 initial Spec review

Pinned diff: `f3897c1f56f872cebee6d3aec857baae7db53e45...9c0e4a67a0fa8f50fa0cc74ead9deb3cc90b2e28`. Reviewed the complete four-file diff and four focused commits against Card045 and the user's full-width source-entry request.

## (a) Missing or partial requirements

No actionable findings. Removing the reserved stable gutter allows nonoverflowing rows to reach the panel's inner edge, fulfilling "Source rows extend to the panel's inner right edge when no scrollbar is needed." Native overflow scrolling remains enabled for longer lists.

Source truncation, flex/grid sizing, counts and removal controls remain intact. Both Sources and Playlists use the same library-list class, satisfying "Shared playlist list styling receives the same gutter correction."

## (b) Unasked scope

No actionable findings. The sole production change removes one CSS declaration. Shared playlist behavior is explicitly requested by the card. There are no component, data, navigation or playback changes.

## (c) Implemented incorrectly

No actionable findings. Native measurements compare actual source row and panel edges and assert no row overflow across the existing Settings scenarios. The parent reports the meaningful pre-fix 15px failure followed by green. Interaction logic is untouched, preserving "Search, selection and removal behavior remain unchanged."

The change uses existing native overflow behavior and does not replace scrolling or expand the panel. Ordered verification/review handoff remains intentionally pending. This read-only review did not rerun the parent's checks or change repository/external state.


## Final Standards

Hard documented violations: none found.

Judgement-call smells: none actionable on the second review of Card 045 from f3897c1f56f872cebee6d3aec857baae7db53e45 to 9c0e4a67a0fa8f50fa0cc74ead9deb3cc90b2e28, reviewed in the full-width-source-rows worktree.

Rechecked the single shared CSS deletion and maintained native measurement/assertion extension. The fix restores full-width rows when no scrollbar is needed while retaining native overflow behavior. It introduces no custom scroller, duplicated harness, application state or error-handling change. Shared Sources/Playlists styling keeps the correction cohesive.

The intended failing edge measurement precedes the minimal implementation, with card documentation and formatting in separate commits. Card 045 records the standard native CSS pattern and interaction-preservation scope. The latest user merge policy overrides older checkout wording. The twelve-smell baseline raises no actionable findings; tool-enforced concerns are excluded.

The parent reports all 337 tests passing. Remaining ordered gate completion and PR handoff are still in progress and are not certified here. No implementation corrections were required after initial review.

Findings: 0 hard violations; 0 judgement-call smells.


## Final Spec

# Card045 final Spec review

Pinned diff: `f3897c1f56f872cebee6d3aec857baae7db53e45...9c0e4a67a0fa8f50fa0cc74ead9deb3cc90b2e28`. Confirmed origin/dev still resolves to this fixed base, then freshly reviewed the complete four-file diff, four commits and Card045. Initial review required no fixes; the head is unchanged.

## (a) Missing or partial requirements

No actionable findings. Removing scrollbar-gutter: stable lets fitting rows reach the panel's inner right edge while retaining native overflow scrolling, meeting "Source rows extend to the panel's inner right edge when no scrollbar is needed."

Existing truncation and flex/grid sizing retain source names, counts and removal controls. Sources and Playlists share the corrected library-list selector, satisfying "Shared playlist list styling receives the same gutter correction."

## (b) Unasked scope

No actionable findings. The only production change deletes one CSS declaration. Native measurements, focused tests and the card directly support the width correction. No data, playback or navigation behavior changes.

## (c) Implemented incorrectly

No actionable findings. The native regression measures actual row/panel right edges and row overflow rather than mirroring the CSS implementation. The parent reports the intended 15px red failure followed by green, and all 337 tests passed. Full ordered gates are still finishing and are not certified complete here.

Search, selection and removal handlers are unchanged, meeting "Search, selection and removal behavior remain unchanged." Native scrollbars still appear as needed, with no custom scrolling engine or forced panel expansion.

This review was read-only and did not duplicate the parent's verification runs. PR handoff remains intentionally pending.


All ordered gates passed with 337 tests and Linux ZIP/DEB packaging.
The native regression first failed with a 15px unused gutter, then passed.
