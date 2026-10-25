# Update status bar review

## Scope

Card 055, based on dev at 47532a8ad000950347e4f6269c48785118e3d9b7. The user gave standing permission for two review cycles on project PRs. This change is a focused presentation update; the updater backend and explicit restart confirmation are retained.

## Initial review

Head: 8a7278848ad7627091fb90e7cdd62934a7c8395a.

Standards: zero findings. Spec: zero findings. The full-width footer, small text pill, visible checking/progress states and accessible details match the requested facelift.

## Verification re-review

The same unchanged runtime/test candidate was checked again. Standards: zero findings. Spec: zero findings. No fixes were required. The footer stays within the full-player shell and is hidden with that shell during compact mode.

## Verification

TDD first reproduced the hidden checking state in f366ab3, then made it visible in da69891. Native/footer and availability-label regressions in 0184595 failed before cdb91bd moved the update UI and reduced it to one text line. Existing installer guidance and error details remain accessible through described-by text.

Ordered local gates passed with one worker: install, lint, strict types, 444 tests, build, package, make and formatting. Native coverage measures 32 material/mode/size combinations and real keyboard focus, rejects corrupt updater bytes and checks explicit restart confirmation. Screenshots with the resting pill were inspected. Cross-platform CI is recorded on the PR.

## CI fixture correction

Linux CI caught a native-input timing race: the fixture could measure focus before queued Tab events finished. Test-only correction a38e38a3dffb04dfff37cda71176bcd8043ed118 focuses the test web contents before keyboard navigation and waits for renderer frames afterward. All real Tab, focus-visible, outline and layout assertions remain in place. Fix re-review reported Standards 0 and Spec 0. Production code is unchanged. The corrected complete local test suite passes all 444 tests.
