# AppImage desktop integration review

Fixed point: 472907ca5a7fae6bb5aa6eee0bad8a4d83f6f7e4 (dev).
Spec: card 047 and the request for automatic first-launch Linux application menu
registration with the real icon and normal desktop pinning, across distributions.

## Reproduction

The released 0.0.2 AppImage creates a muzo-class window but no per-user menu entry.
Its embedded launcher matches that identity. The manually created Cinnamon entry
used a temporary mount executable and had no icon. A disposable XDG/Gio probe
reproduced missing menu discovery; registration with the outer path fixed discovery.

The real-AppImage smoke test fails on the released artifact because first launch
does not register the menu entry, then passes after startup integration. Its
initial cleanup masked the assertion; the harness was corrected and the intended
missing-registration assertion was rerun before the production startup change.

## Initial review

Head: 23d7211eababa2c531a835675e6b0f2776bceb22.
Independent Standards review: zero hard violations and zero actionable smells.
Independent Spec review: two P2 findings.

- An unreadable unrelated desktop shortcut blocked the complete registration scan.
- Standard whitespace around an existing StartupWMClass key bypassed preservation.

## Corrections

Separate red/green regressions prove both failures and fixes. Unreadable unrelated
entries are skipped; unreadable Muzo-specific entries still return a typed storage
failure. Desktop Entry group keys and values are parsed with standard whitespace,
while unrelated groups and comments do not participate in matching.
One initially nested regression was corrected in its own test commit and then
rerun for the intended storage-failure red before applying its implementation.

## Fix re-review

Head: 50f143a47b6fda7a9e6a5a3cd4291ba49cddd4d4.
Fresh independent Standards review: zero hard violations and zero actionable smells.
Fresh independent Spec review: zero findings. Both initial cases were independently
reproduced again, including preservation of the unreadable Muzo-entry failure.

## Verification

Targeted integration suite: 18 tests, including actual Gio discovery/execution,
argument quoting, relocation, concurrent registration, foreign launchers, invalid
paths, storage failure and AppImage/platform guards. Native AppImage smoke tests
use temporary XDG directories and Xvfb; they leave the real desktop untouched.
Full ordered gates and final packaged smoke results are recorded in the PR.
The tests verify the freedesktop/XDG boundary, not every distribution's shell UI.

Final ordered local gates passed: frozen Yarn install, format check, typed lint,
strict types, all 393 tests (375 desktop, six distribution, 12 site), desktop
build/package/Linux ZIP and DEB make. The rebuilt AppImage also passed both
native menu smoke scenarios. Remote CI outcomes are recorded in the PR.
