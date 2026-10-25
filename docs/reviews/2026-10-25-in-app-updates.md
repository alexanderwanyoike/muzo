# Card 049: In-app updates review

Base: `a5c443bffc1f5d77a8ed07e77bcf52b8e750992c`, the reviewed AppImage integration dependency in PR #77. Mini player chrome in PR #78 is independent.

## Initial review

Candidate: `de3803bde398bd345160756f20e6455e91785cfb`.

Standards: zero hard violations and zero actionable Fowler smells. Application, adapter, IPC, UI and release inspection responsibilities were cohesive; expected failures used Results and red/green commits were separate. The reviewer independently identified the packaging concern below and classified it as a Spec issue.

Spec: two findings.

1. P1: electron-builder's prepackaged path skips its afterPack hook that normally writes `resources/app-update.yml`. electron-updater still needs its cache setting. The handwritten development fixture config masked the installed-app failure.
2. P2: discovery checked the four installers but did not require release metadata, so it could announce an incomplete automatic update.

## Corrections

The distribution command now generates the standard fixed-feed config and cache setting inside the Forge payload before builder runs. The native fixture consumes that production-generated config. Installer CI checks the shipped resource in both AppImage and installed NSIS payloads.

Discovery now requires the canonical release JSON, SHA256SUMS and both update YAML files for newer versions. Already installed or older releases do not require updater YAML, preserving compatibility with v0.0.2. Version comparison has one application-layer implementation.

## Fix re-review

Candidate: `00ed9f1881d9c602dd2b1d54577da5ea20803326`.

Standards: zero hard violations and zero actionable smells after a complete final diff review.

Spec: zero findings after a complete final diff review. Both initial issues were fixed; the reviewer confirmed the generated config loads through electron-updater's real cache helper. Explicit download/restart, normal-quit safety, manual platform fallback and same-path replacement remained intact.

The subsequent `b9017e2` commit only simplifies the release-asset arrow expression for lint compliance. Runtime behaviour is unchanged.

## Verification

Ordered local checks are recorded on the card and PR. Native Electron checks reject a corrupt download, retry successfully, retain the old executable until explicit Restart, cancel Restart without installation, execute the verified replacement at the existing path, and measure the footer at 800/1200 pixels in both light and dark modes. The test payload is an executable fixture, not a published AppImage. Packaged installer resource checks are additional to that fixture.
