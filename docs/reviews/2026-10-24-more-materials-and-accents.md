# Card 034 review record

Base: `794dcd410bf95ad88f48cbd81823667b70a02f24`.
Code head for both cycles: `f3303ea93ba9823f87f3e7435a32da33d20bf77e`.
Each cycle used the full three-dot diff/commit list, AGENTS.md and Card 034.
No initial findings required an implementation change.

## Initial Standards

No documented-standard violations or actionable baseline smells found in the pinned Card 034 diff from 794dcd410bf95ad88f48cbd81823667b70a02f24 to f3303ea93ba9823f87f3e7435a32da33d20bf77e.

The change extends the existing material/accent data and CSS token system without adding a dependency, alternate preference model or speculative abstraction. Shared panel chrome consumes the same material token, and the expanded accent control wraps within its existing responsibility. Existing defaults and typed preference handling remain unchanged.

Separate regression commits cover persistence of each new choice and actual native selection, distinct palettes and contrast; focused implementations and formatting follow. The native fixture now derives supported materials from the maintained catalogue instead of duplicating a three-theme branch. The small contrast helper uses the recorded WCAG luminance formula within test fixtures, and the existing geometry assertions continue across 48 combinations.

Card 034 records the established token/persistence pattern and WCAG reference. The three-dot diff contains only personalization despite the reviewed icon ancestry reconciliation. The twelve-smell baseline raises no actionable findings; tool-enforced concerns are excluded. Final ordered gates remain with the parent review.

Findings: 0 hard violations; 0 judgement-call smells.

## Initial Spec

# Spec review: more materials and accents, initial cycle

Pinned diff: `794dcd410bf95ad88f48cbd81823667b70a02f24...f3303ea93ba9823f87f3e7435a32da33d20bf77e`. Reviewed the full diff and commit list against card 034 and the personalization request.

## (a) Missing or partial requirements

No actionable findings in the pinned implementation.

## (b) Unasked scope

No actionable findings. Existing muted-text adjustments and carrying material tokens through panel chrome directly support the requested readable, coherent personalization. The effective diff does not reintroduce icon changes from the reconciled ancestry.

## (c) Apparently implemented requirements done incorrectly

No actionable findings. Carbon fibre, Walnut and Porcelain extend the existing material identifiers and provide complete light/dark surface, text and material tokens. Teal, Rose, Slate, Violet and Ice extend the existing accent controls and both mode palettes, meeting the six-material/ten-accent requirement.

The storage key, original identifiers, validation approach and default Studio/light/Cobalt selection remain unchanged, satisfying "Preserve existing saved preferences and defaults." New App-level tests select each added material/accent and verify restoration after remount. The existing Result-based preference boundary remains intact.

Native Settings coverage now selects materials from the actual catalogue and asserts the applied theme/mode, preventing the previous fixture fallback from silently masking new themes. It checks all 48 layout combinations, distinct material/accent colours, normal/muted text against each surface token, and accent-label contrast. This covers "Verify actual selected material/mode in native layout coverage" and the contrast-check criterion. The larger accent control wraps rather than retaining a fixed height; six previews remain in the shared Settings section.

The parent owns completion of browser inspection at minimum size and final ordered gates. This read-only review inspected pinned source/tests without modifying repository/browser state or duplicating those runs.

## Final Standards

No documented-standard violations or actionable baseline smells found on the second review of the pinned Card 034 diff from 794dcd410bf95ad88f48cbd81823667b70a02f24 to f3303ea93ba9823f87f3e7435a32da33d20bf77e.

Rechecked catalogue-driven theme selection, persisted appearance values, shared chrome tokens and wrapped accent controls. The change extends existing responsibilities without adding a dependency, speculative abstraction or alternate failure-handling path. The native fixture derives supported materials from the maintained catalogue, while a small test-only WCAG helper checks recorded colour tokens across the expanded matrix.

Separate failing persistence and palette regressions precede their implementations, with styling and formatting focused in later commits. Card 034 records the existing token-system pattern and WCAG reference. Reviewed icon ancestry does not expand the actual three-dot personalization diff. The twelve-smell baseline raises no actionable findings; tool-enforced concerns are excluded.

The parent reports all ordered gates passing with 337 tests, 48 native layout cases and all ten accent/text contrast checks at least 4.5. Browser checks covered the new materials at minimum size with no overflow and retained Settings alignment.

Findings: 0 hard violations; 0 judgement-call smells.

## Final Spec

# Spec review: more materials and accents, second cycle

Pinned diff: `794dcd410bf95ad88f48cbd81823667b70a02f24...f3303ea93ba9823f87f3e7435a32da33d20bf77e`. Rechecked the complete changed-file set and the same card 034 requirements. The initial review required no code fix.

## (a) Missing or partial requirements

No actionable findings.

## (b) Unasked scope

No actionable findings. Palette/readability adjustments, preview controls and shared chrome material treatment remain focused on the requested personalization.

## (c) Apparently implemented requirements done incorrectly

No actionable findings on fresh inspection. The material and accent catalogues contain the requested six/ten choices, and all three new materials supply light/dark tokens. Original identifiers, storage key and Studio/light/Cobalt defaults remain unchanged, satisfying "Preserve existing saved preferences and defaults." App regressions retain coverage of selecting and restoring every added choice.

The native fixture resolves the selected theme from the actual catalogue and asserts applied theme/mode for all 48 layout cases. It checks each surface's normal/muted text and all ten accent-label combinations at the required contrast threshold, matching the actual-theme and contrast criteria.

The accent control uses wrapping with a minimum height, allowing the larger palette to remain usable. The parent reports inspected Walnut/dark/Teal, minimum-size Carbon/dark/Violet and Porcelain/light/Rose views with zero overflow, ten swatches in 328px and aligned sections. This supports "Keep appearance controls usable at 800x600 with a wide sidebar."

All 337 tests and ordered formatting, lint, type, build, package and make gates are reported green by the parent. This read-only review did not duplicate those runs or change repository/browser state.


Initial totals: Standards 0, Spec 0. Final totals: Standards 0, Spec 0.

All ordered gates pass with 337 tests. Native layout coverage exercises 48 scenarios and every accent/text contrast combination. Browser QA covered new materials/modes/accents and minimum-size wrapping.
