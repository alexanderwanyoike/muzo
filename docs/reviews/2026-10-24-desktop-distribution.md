# Card 039 review record

Base: `e805aae3cdf57f55468d2b1189d71f7514faa8f2`.
Initial reviewed head: `905c759fc09e890c4080d7ee172902e2f17a3f6d`.
Final reviewed head: `7c7dc5a729a1ed267e60a1f2390534021a8df24b`.
Both axes reviewed the full three-dot diff and atomic commit list in each cycle.

## Initial Standards

Hard documented violations: none found.

Judgement-call smell:

- [P3] Possible Duplicated Code: the identical stable-version regex is independently maintained in packages/distribution/release-contract.ts:54, release-context.ts:23, assemble-release.ts:30 and apps/desktop/electron/distribution/make-distribution.ts:24. These implement the same release contract and can drift. Export one small isStableReleaseVersion guard from the existing contract and reuse it at those boundaries; existing contract/context/assembly/plan tests should stay green in a separate refactoring commit.

Reviewed the pinned diff from e805aae3cdf57f55468d2b1189d71f7514faa8f2 to 905c759fc09e890c4080d7ee172902e2f17a3f6d in the distribution worktree. Pure contract validation, installer inspection, assembly and CLI responsibilities are separated. Expected failures use typed Results; hashing streams large artifacts, handles close in finally, and existing output directories are preserved. Native container constants are named.

Packaging composes the established Forge payload and official prepackaged builder flow instead of replacing runtime identity. Workflows keep native verification separate from assembly and restrict draft creation to verified annotated main tags; no signing/updater scaffolding is introduced. Licence documentation is retained, and the committed Manrope notice matches the installed dependency exactly. The history preserves focused red/green/refactoring phases.

Tool-enforced concerns are excluded. The parent reports ordered gates and real Linux AppImage verification passing; actual Windows/macOS installation/mount checks await PR CI.

Findings: 0 hard violations; 1 judgement-call smell.


## Initial Spec

# Spec review: native desktop distribution, initial cycle

Pinned diff: `e805aae3cdf57f55468d2b1189d71f7514faa8f2...905c759fc09e890c4080d7ee172902e2f17a3f6d` in the dedicated distribution worktree. Confirmed refs/nonempty diff and reviewed all 22 commits against card 039 and the native-download request.

## (a) Missing or partial requirements

No actionable implementation findings. Real Windows installation and both macOS mount/architecture checks are wired into PR CI but await that run; they are not certified complete by this review. Website download presentation remains a separate consumer of the contract.

## (b) Unasked scope

No actionable findings. Separate 0.0.2 preparation supports installer release identity while preserving immutable v0.0.1. No updater, signing claim, dev/main merge, new tag or public release publication is added.

## (c) Apparently implemented requirements done incorrectly

No actionable findings. The platform plan builds the existing Forge payload before electron-builder prepackaged NSIS, arm64/Intel DMG and x64 AppImage creation, satisfying "Build Windows x64 NSIS, macOS arm64/Intel DMG, Linux x64 AppImage from the existing Forge payload." Runtime identity/profile/path code is unchanged, and bundled font notices are retained.

The shared manifest requires all four unique targets, version-correct names, positive sizes, source SHA and hashes. Assembly validates native containers, streams checksums, includes manifest checksums and refuses existing output directories. Native jobs compare main/WASM with Forge originals and verify renderer/notices, with Linux extraction and macOS architecture checks covering actual payloads.

Release jobs require annotated tags whose commits are already on main and whose versions match root/desktop metadata. Packaging explicitly disables publication; the guarded final job creates drafts and does not replace existing releases, meeting "Permit draft release creation only for annotated main tags matching both package versions; never replace tags/releases."

The parent reports all ordered gates, 339 desktop plus six distribution tests and real AppImage extraction passed. This read-only review did not duplicate those runs or change repository/external state.


## Final Standards

Hard documented violations: none found.

Judgement-call smells: none remaining in the pinned Card 039 diff from e805aae3cdf57f55468d2b1189d71f7514faa8f2 to 7c7dc5a729a1ed267e60a1f2390534021a8df24b, reviewed in the distribution worktree.

The initial P3 duplication finding is resolved. The pure release contract now owns isStableReleaseVersion and isReleaseSourceCommit, reused by manifest/context validation, assembly and native packaging. The focused behavior-preserving refactoring retains boundary type narrowing and the existing accepted formats without introducing a generic validation framework.

The remaining design keeps contract rules, native inspection, assembly and CLI concerns cohesive. Expected failures retain typed Results, native resources are preserved through official prepackaged tooling, and installer verification/assembly remain separate from annotated-main-tag draft creation. Existing releases and output directories are not replaced. Established tool/licence records, named container constants and focused red/green/refactoring history remain intact.

The twelve-smell baseline raises no further actionable findings; tool-enforced concerns are excluded. The parent reports the six distribution tests and native-plan test passing after refactoring. Earlier full ordered gates and real Linux AppImage validation passed; final ordered rerun and Windows/macOS installation/mount CI remain with the parent.

Findings: 0 hard violations; 0 judgement-call smells.


## Final Spec

# Spec review: native desktop distribution, second cycle

Pinned diff: `e805aae3cdf57f55468d2b1189d71f7514faa8f2...7c7dc5a729a1ed267e60a1f2390534021a8df24b` in the dedicated distribution worktree. Rechecked the full changed-file set and focused shared-validation refactor against card 039. The initial Spec review required no correction.

## (a) Missing or partial requirements

No actionable implementation findings. Real Windows installation and both macOS mount/architecture jobs remain pending PR CI. The separate website feature will consume published installers rather than exposing unpublished candidates.

## (b) Unasked scope

No actionable findings. Canonical stable-version/source-SHA guards directly support the shared release contract. No public release, new tag, branch merge, updater or signing/notarization claim is introduced.

## (c) Apparently implemented requirements done incorrectly

No actionable findings on fresh inspection. Version and source-commit validation retain the same accepted/rejected values across packaging, manifest parsing, context and assembly. Both application manifests remain 0.0.2; the v0.0.1 tag is untouched, matching "Prepare 0.0.2 in a separate version-only commit, leaving v0.0.1 unchanged."

Four native targets continue packaging the existing Forge payload, with shared versioned filenames, complete-manifest requirements, sizes, streamed SHA-256 and native-container inspection. Runtime identity/profile/path code is unchanged; native verification compares main/WASM and checks renderer/notices.

Annotated-tag/main ancestry and matching-package guards still gate draft creation. Builder publication is disabled and existing releases/output sets are not replaced, preserving "Permit draft release creation only for annotated main tags matching both package versions; never replace tags/releases."

The parent reports the full 345-test ordered baseline and real AppImage extraction/unchanged payload checks passed, followed by all six distribution tests and native-plan regression passing after refactoring. Final-head gates will be repeated before push. This read-only re-review did not duplicate those runs or alter repository/external state.


## Verification

All ordered gates were repeated on the refactored code: Yarn install, formatting,
lint, strict types, 339 desktop tests plus six distribution tests, desktop build,
Linux Forge package and ZIP/DEB make. Red/green commits cover artifact naming,
complete manifests, version/source binding, native plans, release assembly,
checksums and wrong CPU/ELF encoding rejection. The final refactor reuses one
stable-version and source-commit guard without changing behavior.

A real 0.0.2 Linux x64 AppImage (130868704 bytes) was built and extracted. The muzo
executable, desktop identity, renderer, font OFL, main.cjs and SQLite WASM are
present; main/WASM match the Forge payload byte-for-byte. Native CI will verify
Windows silent installation and macOS DMG mounting, architecture, Info.plist and
payload identity after the PR opens. Those remote checks are separate from the
completed local Linux verification.

Initial 0.0.2 artifacts are unsigned unpublished candidates. Draft release creation
requires an annotated immutable main tag matching both package versions, plus a
complete validated release set. No tags, existing releases, dev or main were
changed. The earlier v0.0.1 tag remains untouched.

Totals: initial Standards 0 hard violations/1 P3 smell, Spec 0; final Standards 0,
Spec 0. The shared release-identity guard resolves the initial duplication finding.
