# Mini player chrome review

Fixed point: 472907ca5a7fae6bb5aa6eee0bad8a4d83f6f7e4 (dev).
Spec: card 048 and the request to remove curved mini player ends and move its
icon-only entry control to the large player's top-right, below Close above Volume.

Initial and fresh final independent Standards/Spec reviews at
529420e2a21ccce13e9ee826c2197a81ffb3ce5d returned zero findings on both axes.
The second review rechecked the complete unchanged implementation.

Separate red/green commits cover rectangular windows and icon-only placement.
Native checks cover eight library layouts and sixteen compact/queue layouts in
aluminium/glass light and dark, at normal/minimum sizes. They verify button bounds,
spacing above Volume, zero renderer corner radii, no custom X11 shape, restoration,
queue accessibility and contrast. Existing playback and focus-return tests pass.
Native captures were inspected in /tmp/muzo-mini-chrome/; they use fictional art
and disposable profiles without altering the real desktop.

All ordered local gates passed: frozen Yarn install, format check, typed lint,
strict types, complete tests, desktop build/package and Linux ZIP/DEB make.
AppImage registration and the upcoming updater remain separate feature PRs.
