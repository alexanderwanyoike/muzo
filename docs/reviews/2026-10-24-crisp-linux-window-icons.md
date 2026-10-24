# Card 033 review record

Base: `bc128326b7fb56ebeb92c7e106babb5b608e0aef`.
Initial head: `a78fcaaef90a6737d13c04d26152bea8efb3b34d`.
Final code head: `1b215e705a977a1b12863c9fc142b61702452349`.
Both cycles used the full three-dot diff/commit list and AGENTS.md/Card 033.

## Initial Standards

Hard documented violations: none found.

Judgement-call smell:

- [P3] Possible Mysterious Name in apps/desktop/electron/desktop-icon.test.ts:12-18. The table now supplies full filenames, `"muzo-window.png"` and `"muzo.ico"`, while the parameter remains `extension` and is passed to `path.endsWith(extension)`. Rename it to `filename` or `iconFilename` so its name matches the revised contract.

The exporter remains a focused build tool using Electron's bitmap/PNG API, named geometry and explicit two-colour pixels. Runtime resolution stays in the existing small platform adapter, and the original master/ICO/ICNS blobs are unchanged. The native regression reads the real X11 icon property with bounded process output, validates dimensions/completeness and checks distinct bars after panel-size reduction. Linux CI installs its X11 utility dependency and checks reproducible asset exports through Yarn.

The history separates failing native coverage, the complete-property test correction, focused implementation, build-tool boundary refactoring and formatting. Card 033 records the established Cinnamon/Electron path and small-icon hinting rationale without copying implementation or adding a dependency. Tool-enforced concerns are excluded; final ordered gates remain with the parent review.

Findings: 0 hard violations; 1 judgement-call smell.

## Initial Spec

# Spec review: crisp Linux window icons, initial cycle

Pinned diff: `bc128326b7fb56ebeb92c7e106babb5b608e0aef...a78fcaaef90a6737d13c04d26152bea8efb3b34d`. Reviewed the complete diff and its seven commits against card 033 and the Cinnamon distortion report.

## (a) Missing or partial requirements

No actionable findings in the pinned implementation.

## (b) Unasked scope

No actionable findings. The dedicated Linux rendition, native-property verification and reproducibility wiring directly address small panel rendering. No new dependency or redesign of the approved master is introduced.

## (c) Apparently implemented requirements done incorrectly

No actionable findings. Linux window resolution now selects the shipped 48px `muzo-window.png`; Windows and macOS keep their prior icon paths. The high-resolution PNG, ICO and ICNS are unchanged, satisfying "Preserve the high-resolution launcher, Windows ICO and macOS ICNS artwork." Existing package/launcher configuration also remains unchanged.

The export script builds nine separate blue bars using pixel-aligned two-colour geometry. The native Electron fixture reads the complete actual X11 `_NET_WM_ICON` property, validates its bounds, reconstructs the advertised image, scales it and requires nine distinct centreline runs at both 24px and 32px. This directly covers "Keep the nine waveform bars distinct at 24px and 32px panel sizes" and "Verify the actual X11 _NET_WM_ICON exported by an Electron window."

The existing Yarn icon command now regenerates the small variant, and Linux CI installs the X11 inspection utility and compares regenerated assets against Git. Native Windows/macOS behavior remains platform-limited as before. This supports reproducibility without changing application identity, storage or playback.

The parent reports the intended red-to-green native property result; final ordered gates remain in progress. This read-only review inspected the small PNG and pinned source without changing repository/browser state or duplicating those runs.

## Final Standards

No documented-standard violations or actionable baseline smells remain in the pinned Card 033 diff from bc128326b7fb56ebeb92c7e106babb5b608e0aef to 1b215e705a977a1b12863c9fc142b61702452349.

The initial P3 naming observation is resolved: desktop-icon.test.ts now calls the full filename parameter iconFilename. This is a focused behavior-preserving refactoring commit with the relevant test passing.

The exporter stays within build tooling and uses Electron's existing bitmap/PNG API with explicit pixel geometry. Runtime platform selection remains cohesive, and the approved master/ICO/ICNS assets remain unchanged. Native coverage reads the actual bounded X11 property and verifies nine separate blue bars at 24px and 32px. CI provides xprop and checks repeatable exports through Yarn. Card 033 records the established Cinnamon/Electron path and small-icon hinting rationale without copying implementation or adding a dependency.

The twelve-smell baseline raises no further actionable findings; tool-enforced concerns are excluded. The parent reports 329 tests, install/format/lint/types/build/package passing and zero asset drift after regeneration. Final make completion remains with the parent review.

Findings: 0 hard violations; 0 remaining judgement-call smells.

## Final Spec

# Spec review: crisp Linux window icons, second cycle

Pinned diff: `bc128326b7fb56ebeb92c7e106babb5b608e0aef...1b215e705a977a1b12863c9fc142b61702452349`. Rechecked the full changed-file set and additional test-variable rename. The initial Spec review required no implementation fix.

## (a) Missing or partial requirements

No actionable findings.

## (b) Unasked scope

No actionable findings. Production changes remain limited to selecting the dedicated small Linux icon and reproducibly exporting that asset.

## (c) Apparently implemented requirements done incorrectly

No actionable findings on fresh inspection. Linux selects the 48px pixel-aligned PNG while Windows/macOS and the high-resolution launcher retain the approved artwork and existing paths, satisfying "Preserve the high-resolution launcher, Windows ICO and macOS ICNS artwork."

The fixture still obtains the complete real X11 `_NET_WM_ICON`, validates dimensions/payload length, reconstructs the advertised bitmap and requires nine distinct blue runs after scaling to 24px and 32px. This verifies the actual window property rather than only checking the source file, meeting both native-property and distinct-bar criteria. The final naming correction does not change those assertions.

The Yarn export command and Linux CI regeneration comparison cover the committed small asset's reproducibility. Other runtime responsibilities, package metadata, storage identity, source paths and playback remain unchanged.

The parent reports all 329 tests plus formatting, lint, types, build and package green, unchanged reproducible exports, and the 48px/nine-bar native result. Make remains in progress. This review did not duplicate those runs or change repository/browser state. Final Mint/Cinnamon shell appearance remains a manual check of the user's actual desktop.


Initial totals: Standards 1 (P3 naming heuristic), Spec 0. Final totals: Standards 0, Spec 0.

All ordered gates pass with 329 tests. Native X11 checks read the entire icon property and confirm nine separate bars at 24/32px. Original master and native containers are unchanged; regeneration is deterministic locally.
