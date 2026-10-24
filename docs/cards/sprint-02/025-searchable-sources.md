- Sprint: 02
- Type: Feature
- Status: In review
- Depends on: 024 (PR #51)

# Card 025 - Searchable source list

Sources should work as a compact desktop list when the user has many folders,
with immediate name/path search and visible removal actions instead of pills.

## Acceptance criteria

- [x] Filter names and folder paths case-insensitively, ignoring surrounding spaces.
- [x] Filtering does not change selection, playback or library contents.
- [x] Clear search by button or Escape, restoring the selected row and input focus.
- [x] Announce matching counts and explain unmatched searches.
- [x] Keep source rows in their own scroll area, with navigation/search/artwork fixed.
- [x] Preserve source selection and removal; show counts only once loaded.
- [x] Verify 80 sources at the 800 by 600 minimum and at the normal window size.
- [ ] Complete ordered checks and two review cycles.

## Established pattern

Use React's documented array-filter/list-key pattern, without a new search or
virtualization dependency: https://react.dev/learn/rendering-lists (React, MIT).
Use the native labelled search input described at
https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/input/search.
No external source code is copied. Search is presentation state, so it stays in
the UI rather than adding domain or IPC concepts.

## Verification

LibraryList behavior tests cover name/path search, unchanged selection callbacks,
clear/focus/selection restoration, Escape, filtered removal, empty and unknown
counts. Each new behavior was observed failing before its implementation.
Browser fixture uses 80 disposable local/Dropbox sources and real React views.
At 800 by 600, the list has 108px of visible height and 2560px of content; the
sidebar's scroll height equals its 518px height and album artwork stays visible.
