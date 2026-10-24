# Card 025 review evidence

Base: e670017ce08e57e4c7ab8bc32fde30fefe823291.
Code head: cf20ca7a866aa70c86589d19ba75e8d9213962dd.
Subsequent changes only record review/check completion.

# Initial review

## Standards

No documented-standard violations or actionable baseline smells found in the pinned diff from e670017ce08e57e4c7ab8bc32fde30fefe823291 to cf20ca7a866aa70c86589d19ba75e8d9213962dd.

Filtering and input focus remain local presentation responsibilities in LibraryList, with selection/removal delegated through existing callbacks. The straightforward array filter adds no dependency or speculative domain abstraction. Source-list styling has a cohesive stylesheet. Card 025 records the established React/native-input patterns and licensing. The history separates meaningful search/reset/count regression tests from their implementations.

No CONTRIBUTING or CODING_STANDARDS files were found; docs/agents is absent. Root AGENTS.md, supplied global rules, accepted ADRs and the twelve-smell baseline informed the review. Tool-enforced concerns are excluded; full repository gates and visual verification remain with the parent.

Findings: 0 hard violations; 0 judgement-call smells.

## Spec

# Spec review: searchable source list, initial cycle

Pinned diff: `e670017ce08e57e4c7ab8bc32fde30fefe823291...cf20ca7a866aa70c86589d19ba75e8d9213962dd`. Reviewed the exact diff and its 11 commits.

## (a) Missing or partial requirements

No actionable findings.

## (b) Unasked scope

No actionable findings. Source count handling and compact sidebar/artwork sizing directly support the requested scalable source list. The diff adds no domain, IPC or search dependencies.

## (c) Apparently implemented requirements done incorrectly

No actionable findings. `LibraryList.tsx` implements card lines 13-16: "Filter names and folder paths case-insensitively, ignoring surrounding spaces," "Filtering does not change selection, playback or library contents," clear-by-button/Escape with input focus, and matching-count/unmatched feedback. Filtering remains component state; selection and removal still use the existing callbacks and library identities. Counts are rendered only when defined.

`library-list.css` and `styles.css` satisfy line 17: "Keep source rows in their own scroll area, with navigation/search/artwork fixed." The provided `/tmp/muzo-sources-800x600.png` shows a labelled search field, compact scrollable source rows, removal actions, intact navigation and visible current artwork at the minimum window size. The selected source remains selected across filtering and clearing; the parent App's track view remains outside the filtered list.

Reviewed code and the supplied minimum-size screenshot. The parent's normal-size visual verification and ordered gates were not independently rerun in this read-only review.

Findings: Standards 0, Spec 0; no fixes required.

# Fix re-review

## Standards

No documented-standard violations or actionable baseline smells found on the second review of the pinned Card 025 diff.

The keyboard and clear-button paths share one reset/focus action. Filtering preserves the original source objects and remains local presentation state; existing callbacks retain selection/removal responsibilities. Search/count controls stay outside the source-list scroll container, and focused source styling lives in its own stylesheet. These choices fit the repository's KISS, cohesion and inward-dependency rules without adding unnecessary abstractions.

Rechecked the focused test-before-implementation history. The twelve-smell baseline raises no actionable findings. Tool-enforced concerns are excluded; repository gates and rendered-layout verification remain with the parent.

Findings: 0 hard violations; 0 judgement-call smells.

## Spec

# Spec review: searchable source list, second cycle

Pinned diff: `e670017ce08e57e4c7ab8bc32fde30fefe823291...cf20ca7a866aa70c86589d19ba75e8d9213962dd`.

## (a) Missing or partial requirements

No actionable findings.

## (b) Unasked scope

No actionable findings. The source count correction and sidebar sizing remain directly related to a searchable list that accommodates many sources.

## (c) Apparently implemented requirements done incorrectly

No actionable findings after fresh inspection of the pinned component and layout diff. The name/path substring filter trims and normalizes case, without calling selection, removal or playback handlers. Button and Escape clearing restore the full list and input focus; existing identity-based selection remains intact. Loaded counts, matching-count announcements and unmatched feedback satisfy card 025 lines 13-18.

The list owns its scroll area while navigation, search and artwork remain outside it, satisfying "Keep source rows in their own scroll area, with navigation/search/artwork fixed." Independently inspected both supplied screenshots, including `/tmp/muzo-sources-normal.png`; the compact rows and artwork are visible at normal and minimum sizes.

The parent's browser observations additionally establish that source 80 is reachable with list scrollTop 2452 while sidebarScroll remains 0 and artwork stays on screen. Path search finds `/Archive/79`; unmatched search feedback and Escape restore all 80 sources and the previous selection. This supports "Verify 80 sources at the 800 by 600 minimum and at the normal window size." The review made no repository or browser changes and did not duplicate the parent's gate run.

Findings: Standards 0, Spec 0; no fixes required.

Ordered install, format, lint, typecheck, 279 tests, build, package and make passed.
Browser checks used 80 fixture sources, including minimum size, last-row scroll,
name/path search, unmatched feedback and Escape/selection restoration.
