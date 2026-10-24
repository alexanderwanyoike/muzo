- Sprint: 02
- Type: Fix
- Status: In review
- Depends on: 030 (PR #57)

# Card 031 - Consistent desktop Settings

The user reported uneven section widths and scattered controls, and explicitly
requested the frontend-design skill. Use restrained desktop preferences styling:
one centred content column, aligned section headers/bodies, consistent controls,
clear grouping and responsive action rows across all existing materials/modes.

## Acceptance criteria

- [x] Give Appearance, Dropbox, Library folders and Indexed sources equal widths/edges.
- [x] Share section heading, description, body spacing and surface treatment.
- [x] Align field labels, inputs and actions; use consistent standard control heights.
- [x] Wrap long source paths/actions without clipping at minimum window width.
- [x] Preserve source-setup focus, folder browsing, connection/sync, appearance and removal.
- [x] Verify connected/disconnected Dropbox, long paths, all materials in light/dark, and minimum 800x600.
- [x] Complete red-first layout regression tests, ordered gates and two review cycles.

## Established pattern

Reviewed libadwaita's PreferencesGroup pattern of related preference rows under a
shared title/description and optional header accessory:
https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.PreferencesGroup.html
This informs grouping and alignment. No GTK dependency or external implementation
is copied. React composition provides one small SettingsSection used by the four
existing sections. Existing material/colour tokens and Manrope typography remain
the visual language.

## Verification

The first native-renderer regression failed because Dropbox measured 451.64px
against Appearance at 850px. A later field-edge regression observed an 8px
mismatch. Separate red/green commits fix both. Native regression coverage now
checks 24 combinations: three materials, two modes, connected/disconnected
Dropbox and 1280/800px widths with the widest sidebar preference. It asserts
matching section edges/heading positions, 20px gaps, common field edges, standard
control heights and no horizontal overflow. Windows/macOS CI includes this test.

All ordered gates pass: install, format check, lint, typecheck, 329 tests, build,
package and make. Browser inspection confirmed all four sections at 469px wide
with the same 298px left edge at 800x600, long source paths and compact actions.
The dark minimum-width Dropbox picker measured 435px with zero overflow;
selection worked and closing restored Browse focus. Light/dark materials were
inspected. Two review cycles have zero findings on both axes.
See [review record](../../reviews/2026-10-24-consistent-settings-layout.md).
