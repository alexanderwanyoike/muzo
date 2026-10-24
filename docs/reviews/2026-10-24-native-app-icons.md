# Card 026 review evidence

Base: 33738ccb1849069027c923158cadbe2055222c18.
Initial code head: 3ff9c28c58635eaf30384be0fa5d71135f73e431.
Final code head: 183137d892326af213192a26ba51168e6a0b5171.
Subsequent changes only record review/check completion.

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
