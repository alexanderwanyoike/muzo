# Card 029 review record

Base: `32fecf7e7c3f3d2d3eb3eb16fead6d88df400670`.
Initial head: `c685e7ef7dca077f84deafb1ab5a33e0443ef927`.
Final code head: `ebefb2b44e7be05ea500cb8b948ee328131519cb`.

Each cycle reviewed the three-dot diff and full commit list against the card and AGENTS.md. User authorization for two review cycles applies.

## Initial Standards

No documented-standard violations or actionable baseline smells found in the pinned Card 029 diff from 32fecf7e7c3f3d2d3eb3eb16fead6d88df400670 to c685e7ef7dca077f84deafb1ab5a33e0443ef927.

Panel presentation, splitter interaction, layout state and preference persistence have focused modules. Pure width constraints remain simple named functions, expected storage failures return typed Results, and the hook handles them explicitly while preserving session usability. Pointer capture and keyboard paths use the same commit operation without introducing a general layout framework.

The history separates meaningful panel, persistence, bounds, keyboard, pointer and reset regressions from their focused implementations, with later refactoring and styling. Card 029 records the established React panel and WAI-ARIA patterns and explains the narrow valid-role ESLint configuration adjustment. No domain/infrastructure dependencies leak across the new UI responsibilities.

The twelve-smell baseline raises no actionable findings. Tool-enforced concerns are excluded. The parent reports full ordered gates passing with 319 tests, packaging and make. A cancellation-after-viewport-clamp behavior edge was passed separately to the Spec axis.

Findings: 0 hard violations; 0 judgement-call smells.

## Initial Spec

# Spec review: resizable Sources panel, initial cycle

Pinned diff: `32fecf7e7c3f3d2d3eb3eb16fead6d88df400670...c685e7ef7dca077f84deafb1ab5a33e0443ef927`. Reviewed the exact diff and its 21 commits against card 029.

## (a) Missing or partial requirements

No additional actionable findings.

## (b) Unasked scope

No actionable findings. The panel framing, setup shortcut, width preferences and focused splitter accessibility configuration directly support the requested Sources sidebar.

## (c) Apparently implemented requirements done incorrectly

- [P2] Preserve the preferred width when cancelling a drag in a temporarily narrowed window. `apps/desktop/src/sidebar-splitter.tsx:29-35` restores `active.startWidth`, which is the displayed, clamped width; `use-sidebar-layout.ts:36` sends that value directly to the preferred-width state. Example: choose 360px, narrow the window to 800px so the sidebar displays 280px, begin a drag, then press Escape. Cancellation replaces the preference with 280px, so widening the window no longer restores 360px. This violates the combined requirements "Double-click to reset and Escape to cancel a drag" and "Remember width between launches and preserve it across temporary window narrowing." Keep drag preview separate from the stored preference, discard it on cancellation, and cover narrowing before a cancelled drag in the integration test. The existing test covers cancellation and narrowing separately.

The remaining implementation retains source selection/search/removal and playback state, provides Add source navigation with focus, and implements pointer capture, keyboard movement and width constraints. The parent reports 80-source light/dark/minimum-window browser checks and all 319 tests plus packaging passed. This read-only review did not duplicate those gates.

## Fix re-review Standards

No documented-standard violations or actionable baseline smells found in the updated pinned Card 029 diff from 32fecf7e7c3f3d2d3eb3eb16fead6d88df400670 to ebefb2b44e7be05ea500cb8b948ee328131519cb.

The fix gives transient drag width its own draft state while retaining the preferred width separately. Preview updates the draft, commit saves the chosen width, and cancellation clears the draft through an explicit callback. This keeps layout state ownership in the hook and pointer interaction in the splitter, without adding a generic state abstraction.

The focused cancellation-after-narrowing regression was committed separately before the minimal implementation. It verifies that cancellation restores the clamped display and later widening recovers the original preference. The existing persistence, keyboard, pointer-capture, bounds and panel responsibilities remain cohesive, and expected storage failures retain typed Result handling.

The initial standards assessment otherwise stands, including recorded established patterns and the narrow valid-role ESLint configuration. The twelve-smell baseline raises no actionable findings; tool-enforced concerns are excluded. Final gate completion remains with the parent review.

Findings: 0 hard violations; 0 judgement-call smells.

## Fix re-review Spec

# Spec review: resizable Sources panel, fix cycle

Pinned diff: `32fecf7e7c3f3d2d3eb3eb16fead6d88df400670...ebefb2b44e7be05ea500cb8b948ee328131519cb`.

## Initial P2 verification

Resolved. Card 029 requires "Double-click to reset and Escape to cancel a drag" and "Remember width between launches and preserve it across temporary window narrowing."

`use-sidebar-layout.ts` now keeps transient drag width in `draft`, separate from `preferred`. Preview changes the draft; successful commit updates/persists the preference; cancellation discards only the draft. `SidebarSplitter` invokes the explicit cancellation callback for Escape, pointer cancellation and lost capture. Restoring a wider viewport after a cancelled narrow-window drag therefore recovers the original preference.

The new App integration regression combines the previously separate cases: choose 360px, narrow to an 800px window, drag, Escape, verify 280px while constrained, then widen and require 360px. The parent reports this test first failed for the intended 280-versus-360 discrepancy before the focused implementation commit.

## (a) Missing or partial requirements

No actionable findings in the pinned implementation.

## (b) Unasked scope

No actionable findings. Separating transient resize state directly repairs the required cancellation and width-preservation behavior.

## (c) Apparently implemented requirements done incorrectly

No additional actionable findings. The full slice retains panel search/selection/removal and playback behavior, focused local/Dropbox setup navigation, keyboard and pointer resizing, bounded widths and typed preference failures. The accessible separator remains labelled and reports its controlled pane and current limits.

The parent reports the initial 319-test gate run, packaging and 80-source browser verification at minimum size and in both themes passed. Final post-fix gates remain the parent task's responsibility. This read-only review inspected the pinned changes and did not alter repository or browser state.


Initial totals: Standards 0, Spec 1 (P2). Final totals: Standards 0, Spec 0.

All ordered gates passed: install, format check, lint, typecheck, 320 tests, build, package and make. Browser verification covered 80 sources, light/dark, pointer/keyboard/reset, direct setup focus, and minimum 800 by 600.
