# Card 026 review evidence

Base: 33738ccb1849069027c923158cadbe2055222c18.
Initial code head: 3ff9c28c58635eaf30384be0fa5d71135f73e431.
Earlier pre-waveform code head: 183137d892326af213192a26ba51168e6a0b5171.
That stage was followed by the user-requested waveform revision below.

# Initial review

## Standards

No documented-standard violations or actionable baseline smells found in the pinned diff from 33738ccb1849069027c923158cadbe2055222c18 to 3ff9c28c58635eaf30384be0fa5d71135f73e431.

Runtime icon configuration stays in a small Electron-boundary module with a narrow injected Dock dependency. Forge retains one extension-free asset stem; Linux launcher configuration uses the same master. Build-time conversion uses the established png2icons CLI, with its MIT notice, exact generation prompts and reproducible Yarn command recorded beside the assets. No application-level expected failure flow is introduced.

The added CI job follows the existing pinned-action, Yarn and bounded-run patterns while exercising native platform decoding and packaging. The commit sequence separates failing asset/path/Dock/package contract tests from focused implementations and subsequent formatting. Native decoding coverage supplements those contracts.

Tool-enforced concerns are excluded. Ordered repository gates and target-platform CI outcomes remain with the parent review.

Findings: 0 hard violations; 0 judgement-call smells.

## Spec

# Spec review: native app icons, initial cycle

Pinned diff: `33738ccb1849069027c923158cadbe2055222c18...3ff9c28c58635eaf30384be0fa5d71135f73e431`. Inspected the exact diff and its 13 commits.

## (a) Missing or partial requirements

No actionable implementation findings. Native Windows/macOS CI is explicitly pending in card 026, rather than claimed complete.

## (b) Unasked scope

No actionable findings. Asset conversion, provenance, runtime integration and platform CI directly support the requested Windows/Linux/macOS app identity.

## (c) Apparently implemented requirements done incorrectly

No actionable findings. The pinned PNG inspected here matches the committed blob. Its blue M/audio-wave silhouette on a black rounded tile follows the user's latest direction and card lines 8-10: "rejected the metallic design and requested blue, black and flat." The old metallic artwork is not shipped. Faint generated surface variation is visible, but it does not introduce metallic depth or an additional design motif.

`desktop-icon.ts` resolves PNG/ICO siblings from both development and packaged entrypoint directories; `main.ts` passes the path to initial and recreated windows. The macOS development Dock is set separately while packaged applications retain their bundle identity. Forge config supplies the extension-free package icon and explicit DEB PNG, matching "Configure Forge icons and the Linux DEB launcher icon."

Format contracts cover Windows small sizes and macOS through 1024; the converter is pinned, exports use Yarn, and its licence/provenance are committed. Native integration exercises decoding and window creation, with Windows/macOS packaging added to CI. Linux success is reported by the parent. Target-platform CI and manual OS-shell appearance remain documented verification work and were not independently executed here.

Findings: Standards 0, Spec 0.

# Fix re-review

## Standards

No documented-standard violations or actionable baseline smells found in the final pinned Card 026 diff from 33738ccb1849069027c923158cadbe2055222c18 to 183137d892326af213192a26ba51168e6a0b5171.

The final launcher correction stays within existing Forge DEB configuration: productName is Muzo and genericName is Music player. Its two focused regression assertions were committed separately before the minimal configuration fix. This preserves cohesive packaging responsibility and adds no abstraction or application failure path.

The remaining diff retains the previously reviewed runtime-relative assets, platform-limited development Dock override, shared startup/activation icon path, established converter CLI, exact MIT notice and reproducibility checks. No standards regressions or actionable twelve-baseline smells were introduced. Tool-enforced concerns are excluded.

The parent owns final ordered checks and native Windows/macOS CI verification; previous Linux ordered gates passed before this configuration correction.

Findings: 0 hard violations; 0 judgement-call smells.

## Spec

# Spec review: native app icons, final fix cycle

Pinned diff: `33738ccb1849069027c923158cadbe2055222c18...183137d892326af213192a26ba51168e6a0b5171`.

## (a) Missing or partial requirements

No actionable implementation findings. Windows/macOS CI remains explicitly pending until the PR push, consistent with the unchecked native-platform verification criterion.

## (b) Unasked scope

No actionable findings. The focused DEB launcher-name correction supports card 026's "consistent app identity on Windows, Linux and macOS" and "Configure Forge icons and the Linux DEB launcher icon."

## (c) Apparently implemented requirements done incorrectly

No actionable findings after reviewing the final full diff and the two additional test/fix commits. `forge.config.cjs` now provides DEB `productName: "Muzo"` and `genericName: "Music player"`, correcting the inherited workspace package name in the launcher. The adjacent packaging contract requires both values. The icon path and asset bytes are unchanged; the parent reports the DEB's installed icon hash matches the committed PNG.

The runtime still resolves assets correctly from source `electron` and packaged `dist-electron`, and passes the icon to initial and activation-created windows. Development-only macOS Dock configuration preserves the packaged bundle icon. Committed native containers cover Windows small sizes and macOS artwork through 1024; root assets remain included by Forge. Yarn exports, licensing and provenance are present, and the design remains the requested blue/black flat mark.

The parent reports prior ordered local gates passed with 285 tests, real native icon decoding/window creation and package/make. This review inspected the final source without rerunning gates. Final checks and native Windows/macOS CI remain the parent task's responsibility; manual target-shell appearance remains a documented verification limit.

Findings: Standards 0, Spec 0.

## Verification

All ordered local gates passed with 285 tests. Native Electron decoded the Linux
icon and created a window. Source assets were found in the packaged application.
The extracted DEB launcher icon matched the source SHA-256, and repeated ICO/ICNS
exports matched byte for byte. Actual DEB inspection caught the inherited
scoped package label; red assertions preceded the focused Muzo launcher-name
fix. The final fix re-review inspected that correction. Browser checks covered
16, 32, 64 and 128 pixel icons on light/dark backgrounds. Native Windows/macOS CI
runs on the PR; target OS shell appearance still needs manual testing.

# Waveform revision review cycles

Updated code head: 05f3ffd9d79fb496dc73d21e45f09ecd47167239.
The user requested waveform bars in place of the continuous M. The revision also
prepares Electron once before parallel tests following the first Linux CI failure.

# Waveform initial review

## Standards

No documented-standard violations or actionable baseline smells found in the updated pinned diff from 33738ccb1849069027c923158cadbe2055222c18 to 05f3ffd9d79fb496dc73d21e45f09ecd47167239.

The waveform revision changes the asset family and records its provenance without expanding runtime responsibilities. Native packaging and platform icon configuration retain the previously reviewed small Electron-boundary design, established converter CLI, licence and reproducibility checks.

The test preparation change addresses observed CI failure using Electron's own entry point. The installed Electron 43 loader confirms synchronous installation on first require; running that entry point once before parallel Vitest workers is a focused coordination fix. Both normal and watch scripts share the same Yarn preparation command. Native assertions remain intact, and no custom installer or suppressed failure path is introduced.

The prior test-first asset, runtime and packaging history remains preserved. Tool-enforced concerns are excluded. The parent reports the previous Windows/macOS native decoding/package jobs passed; updated complete gates and CI outcomes remain with the parent.

Findings: 0 hard violations; 0 judgement-call smells.

## Spec

# Spec review: waveform native icon, initial revision cycle

Pinned diff: `33738ccb1849069027c923158cadbe2055222c18...05f3ffd9d79fb496dc73d21e45f09ecd47167239`.

## (a) Missing or partial requirements

No actionable implementation findings. Fresh native-platform verification remains pending and is unchecked in card 026.

## (b) Unasked scope

No actionable findings. Preparing Electron before parallel native tests addresses the observed fresh-install race introduced by exercising two real Electron integrations. The card explicitly requires: "Prepare the binary once before Vitest starts, using Electron's own entry point. Keep the native tests and their assertions intact; watch mode uses the same preparation."

## (c) Apparently implemented requirements done incorrectly

No actionable findings. Independently inspected the PNG and confirmed its blob matches the pinned asset. Nine distinct blue rounded waveform bars form two peaks and a central trough on the black rounded tile, satisfying the revised direction to "replace the continuous M with a waveform style." The existing runtime paths, first/recreated-window handling, development Dock override and Forge/DEB integration continue to use the regenerated sibling assets.

The real native tests, platform-specific configuration and container contracts remain intact. `test` and `test:watch` both run the same `electron:prepare` command before starting Vitest. The installed Electron entry point synchronously ensures its executable exists, so preparation finishes before worker startup without weakening test assertions.

The parent reports Windows/macOS native checks passed for the prior asset. That does not establish the revised containers' final platform results; fresh CI and ordered gates remain with the parent. No repository or browser changes were made by this review.

Findings: Standards 0, Spec 0.

# Waveform fix re-review

## Standards

No documented-standard violations or actionable baseline smells found on the second review of the full pinned diff from 33738ccb1849069027c923158cadbe2055222c18 to 05f3ffd9d79fb496dc73d21e45f09ecd47167239.

The waveform assets retain one master and reproducible native exports, with updated generation provenance and an exact copy of the converter's MIT notice. Runtime icon resolution, development-only macOS Dock behavior and shared initial/reactivated-window paths retain focused Electron-boundary responsibilities. Forge keeps native package icons and the Muzo Linux launcher identity together.

Normal and watch tests both await the same Yarn preparation command before parallel workers. Delegating first-use installation to Electron's existing loader directly addresses the observed race without weakening native coverage or adding a custom installation layer. Existing test-first contract history and focused subsequent fixes remain intact.

Tool-enforced concerns are excluded. The parent reports all 285 tests passed again; final build/package/make completion and updated CI remain with the parent.

Findings: 0 hard violations; 0 judgement-call smells.

## Spec

# Spec review: waveform native icon, second revision cycle

Pinned diff: `33738ccb1849069027c923158cadbe2055222c18...05f3ffd9d79fb496dc73d21e45f09ecd47167239`.

## (a) Missing or partial requirements

No actionable implementation findings. Fresh Windows/macOS CI remains unchecked rather than being inferred from the previous asset's successful run.

## (b) Unasked scope

No actionable findings. Native-binary preparation directly addresses the observed parallel fresh-install test failure while preserving the card's real-platform verification requirement.

## (c) Apparently implemented requirements done incorrectly

No actionable findings after fresh inspection of the complete pinned file set, test commands and native-container assertions. The revised artwork implements "blue waveform bars on a black rounded tile, with transparent corners" and the requested two-peak/central-trough suggestion of an M. Independently inspected `/tmp/muzo-waveform-icon-sizes.png`; the waveform remains recognizable at the supplied 16, 32, 64 and 128 pixel previews on light/dark backgrounds.

The PNG/ICO/ICNS siblings retain the established runtime and packaging integration for Windows, Linux and macOS. Native format contracts still require small Windows sizes and macOS artwork through 1024. The native integration still asserts successful real decoding, 16/32/256 output dimensions and window creation.

Both normal and watch test scripts run Electron's synchronous preparation before starting Vitest, meeting "Prepare the binary once before Vitest starts, using Electron's own entry point." No native assertions were removed. The parent reports all 285 tests, formatting, lint and type checks passed for this revision; remaining ordered gates and platform CI stay with the parent task. Historical review wording will be qualified when the new reports are recorded.

Findings: Standards 0, Spec 0.

All ordered local gates passed again with 285 tests on the waveform revision.
Native Windows/macOS checks passed on the preceding asset revision; the latest
asset and Electron preparation receive fresh three-platform CI on the PR.

## Final native CI

[Run 40094628589](https://github.com/alexanderwanyoike/muzo/actions/runs/40094628589)
passed all jobs on waveform code/documentation head
1903c19c409708c653352699915c80fa51458232:
Linux full verification, Windows native decoding/package, and macOS native
decoding/package. No native test assertions were skipped. The final handoff
commit only records these results. All feature PRs now target dev under the
updated user instructions, with prerequisite merge order #47 through #53.
