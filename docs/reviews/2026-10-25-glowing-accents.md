# Glowing accents review

## Scope

Card 053, based on dev at aab92cdb5b639ff1b6c2c0aa8122f21655ce214b. Pulse remains independent in PR 84. The user gave standing permission for two review cycles on project PRs.

## Initial review

Head: 438313f824916db3792e10c141b69ca113cc2959.

Standards: zero actionable findings. Spec: one P2 finding. The sidebar resize handle retained the bright accent fill for its keyboard-focus indicator, giving Glow green only 1.65:1 contrast against Studio's light sidebar.

## Fix re-review

Head: 5c41e519e72d54d61ccbda2dc7bc3264c90c15ad.

Standards: zero findings. Spec: zero findings.

Native regression c722de9 reproduced the contrast defect after actual Tab focus, then production correction 59f785a changed the indicator to accent ink. Focus contrast now passes across every Glow colour, material, mode and tested window size.

## Verification

Local ordered gates passed with one worker: install, lint, strict types, 441 tests, build, package, make and formatting. Twelve persistence cases cover all colours in light and dark mode. Native checks measure every palette's foreground roles and actual production playback controls across all materials, both modes and full/compact contexts, including Classic restoration and forced-colour fallback.

Windows/macOS CI runs the native test list with one worker because those fixtures share the host window server and keyboard focus. Linux fixtures use separate Xvfb displays. This CI scheduling setting matches the locally verified single-worker configuration and does not change application behaviour.
