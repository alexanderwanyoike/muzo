# Card 030 review record

Base: `0950aed78196c63dceee564024f0ad440178ff16`.
Initial head: `1f6af4cec5190f789042dd7c8a2f11cd0e420628`.
First fix head: `b946b4f15aa9eb2d4504f1b65479e64fd5a5055c`.
Completed code head: `e780d0340ad387d73567a163c924bb085393ed8e`.

Every cycle reviewed the three-dot diff and full commit list against Card 030,
AGENTS.md and the user request. The user's prior review authorization applies.

## Initial Standards

No documented-standard violations or actionable baseline smells found in the pinned Card 030 diff from 0950aed78196c63dceee564024f0ad440178ff16 to 1f6af4cec5190f789042dd7c8a2f11cd0e420628.

Shared panel and search components now have two concrete consumers, satisfying the repository's KISS rule without speculative abstraction. Playlist navigation, creation form and playlist state/mutations retain focused responsibilities. The portal preserves one playlist state owner while placing navigation in the existing sidebar. Expected API rejections become typed Results at the UI boundary and are handled explicitly; creation retains its draft on failure.

The commit history separates meaningful selection, placement, search, creation and navigation regressions from focused implementations, with later shared-component and Result refactoring and styling. Existing queue/source-removal tests now exercise the actual panel navigation. Card 030 records the established React portal pattern, licence and independent implementation.

The twelve-smell baseline raises no actionable findings. Tool-enforced concerns are excluded. The parent reports 324 passing tests and browser coverage of 40 playlists at minimum window size; final ordered gate completion remains with the parent.

Findings: 0 hard violations; 0 judgement-call smells.

## Initial Spec

# Spec review: Playlists sidebar panel, initial cycle

Pinned diff: `0950aed78196c63dceee564024f0ad440178ff16...1f6af4cec5190f789042dd7c8a2f11cd0e420628`. Inspected the exact diff and its 17 commits against card 030.

## (a) Missing or partial requirements

No additional actionable findings.

## (b) Unasked scope

No actionable findings. The shared panel/search frame has concrete Sources and Playlists consumers, and the retained workspace directly supports selection/search continuity across navigation.

## (c) Apparently implemented requirements done incorrectly

- [P2] Coordinate playlist creation with navigation-triggered list loads. `apps/desktop/src/playlist-panel.tsx:65-68` unconditionally replaces playlists when an active-view load resolves, while lines 81-89 append/select a successful creation independently. Submit New playlist, navigate to Songs and back while creation is pending, then let creation succeed before the return-view list snapshot resolves. The late snapshot drops the newly created row and selected detail falls back to the first playlist. The inverse response ordering can also append a duplicate if the refreshed list already includes the new playlist. This violates "Create a playlist from New playlist, select it, and retain drafts on expected failure" and the requirement to show only the selected playlist's tracks. Coordinate load generations with mutations, reconcile by playlist identity, and add deferred-response navigation regressions for both orderings.

The remaining implementation provides the shared framed/searchable/scrollable navigator, one selected playlist detail, consistent saved divider width and accessible labelling. Existing entry artwork, unavailable rows and queue callbacks are retained. The parent reports all 324 tests and 40-playlist/minimum-window browser checks passed; this read-only review did not duplicate those runs.

## First fix re-review Standards

No documented-standard violations or actionable baseline smells found in the updated pinned Card 030 diff from 0950aed78196c63dceee564024f0ad440178ff16 to b946b4f15aa9eb2d4504f1b65479e64fd5a5055c.

The fix retains loaded playlist data during refresh and uses a local revision guard to prevent an older snapshot from overwriting completed edits. Creation reconciles by persisted playlist identity before selecting the result. These responsibilities remain with the existing playlist state owner; no speculative synchronization abstraction or cross-layer dependency is introduced.

Separate failing regressions cover creation-first data loss and refresh-first duplication, followed by focused implementation commits and formatting. The shared frame/search still have two concrete consumers, form failures retain typed Result handling and drafts, and navigation/creation/detail components remain cohesive. Existing queue and removal coverage is maintained.

The initial standards assessment otherwise stands. Card 030 records the established React portal pattern and licensing. The twelve-smell baseline raises no actionable findings; tool-enforced concerns are excluded. Final ordered gates remain with the parent review.

Findings: 0 hard violations; 0 judgement-call smells.

## First fix re-review Spec

# Spec review: Playlists sidebar panel, fix cycle

Pinned diff: `0950aed78196c63dceee564024f0ad440178ff16...b946b4f15aa9eb2d4504f1b65479e64fd5a5055c`.

## Initial P2 verification

The original newly-created playlist loss/duplication is resolved. Successful creation upserts by persisted ID, stale list snapshots are ignored after completed edits, and deferred-response regressions cover both creation/list response orders while preserving the existing playlist.

## (a) Missing or partial requirements

No additional findings beyond the refresh coordination defect below.

## (b) Unasked scope

No actionable findings.

## (c) Apparently implemented requirements done incorrectly

- [P2] Retain independent catalogue changes when invalidating a refresh after a playlist edit. `apps/desktop/src/playlist-panel.tsx:62-68` now drops the entire list response whenever any mutation increments `revision`, without retrying or reconciling other playlists. Start playlist creation, navigate to Sources and remove a source, then return to Playlists while creation remains pending. The return-view load contains the newly unavailable entries. If creation completes first, the revision mismatch discards that load and the retained old playlist data continues showing the deleted source's tracks as playable. This violates "Preserve entry artwork, playback/queue identity, removal, reordering and unavailable entries." Revalidate after discarding a superseded snapshot, or reconcile identity-specific mutation results without losing independent refreshed catalogue data. Add a delayed creation/source-removal refresh regression proving the new playlist remains selected and removed-source entries become unavailable.

The remainder continues to provide one selected detail view, shared sidebar materials/search, retained selection/filter state and the existing saved divider. The parent reports a 41-playlist light/dark/minimum-window browser pass and final gates in progress. This read-only review inspected the pinned implementation without modifying repository or browser state.

## Completed re-review Standards

No documented-standard violations or actionable baseline smells found in the final reviewed Card 030 diff from 0950aed78196c63dceee564024f0ad440178ff16 to e780d0340ad387d73567a163c924bb085393ed8e.

The additional fix requests a fresh collection when a completed edit supersedes a pending snapshot. It preserves the existing single playlist state owner and keeps independent source-availability updates from being discarded. The load trigger remains local UI coordination rather than introducing a speculative synchronization abstraction. Typed Result handling and identity-based creation reconciliation remain unchanged.

A separately committed failing regression exercises source removal overlapping playlist creation; the focused fix follows it, with formatting separate. Both earlier completion-order regressions remain maintained. Shared panel/search components retain two concrete consumers, and form, navigator and playlist mutation responsibilities remain cohesive.

The initial standards assessment otherwise stands, including the recorded established React portal pattern and licensing. The twelve-smell baseline raises no actionable findings; tool-enforced concerns are excluded. Final ordered gates and platform verification remain with the parent review.

Findings: 0 hard violations; 0 judgement-call smells.

## Completed re-review Spec

# Spec review: Playlists sidebar panel, additional fix cycle

Pinned diff: `0950aed78196c63dceee564024f0ad440178ff16...e780d0340ad387d73567a163c924bb085393ed8e`.

## Prior findings verification

Both P2 findings are resolved in the pinned implementation. Creation upserts by persisted ID, avoiding duplicate rows when an overlapping list already includes the new playlist. Successful creation remains selected rather than being overwritten by an older snapshot, satisfying "Create a playlist from New playlist, select it, and retain drafts on expected failure."

When a completed edit supersedes an in-flight list load, the effect now requests another collection refresh instead of dropping independent catalogue changes permanently. The next load captures the current edit revision and reconciles the complete collection. Effect cleanup still ignores obsolete responses after navigation. This preserves source-removal availability updates while retaining the created playlist, satisfying "Preserve entry artwork, playback/queue identity, removal, reordering and unavailable entries."

The new delayed-response regression combines pending creation, a source-removal refresh and an older response. It requires the removed-source track's play control to become disabled and the created playlist to appear exactly once. Both original creation/list response-order regressions remain in place and now supply the subsequent fresh snapshot.

## (a) Missing or partial requirements

No actionable findings in the pinned implementation.

## (b) Unasked scope

No actionable findings. The additional refresh revision directly repairs the identified retained-workspace race.

## (c) Apparently implemented requirements done incorrectly

No additional actionable findings after inspecting the final full file set and focused correction. Shared framing/search, one selected detail view, persistent divider width and accessible labelling remain intact. Expected creation failures preserve drafts and existing queue/artwork/removal/reorder handlers remain connected.

The parent owns final ordered gates and browser verification. This read-only review made no repository or browser changes and did not duplicate those runs.


Initial totals: Standards 0, Spec 1 (P2). First fix re-review: Standards 0, Spec 1 (P2).
Completed re-review: Standards 0, Spec 0. Both P2 findings are resolved.

All ordered gates passed: install, format check, lint, typecheck, 327 tests,
build, package and make. Browser QA covered 40/41 playlists, light/dark materials,
minimum size, independent scrolling, search/selection, creation/focus, entry
reorder/removal and the shared Sources/Playlists divider.
