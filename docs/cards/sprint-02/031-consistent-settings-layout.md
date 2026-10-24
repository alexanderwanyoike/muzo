- Sprint: 02
- Type: Fix
- Status: In progress
- Depends on: 030 (PR #57)

# Card 031 - Consistent desktop Settings

The user reported uneven section widths and scattered controls, and explicitly
requested the frontend-design skill. Use restrained desktop preferences styling:
one centred content column, aligned section headers/bodies, consistent controls,
clear grouping and responsive action rows across all existing materials/modes.

## Acceptance criteria

- [ ] Give Appearance, Dropbox, Library folders and Indexed sources equal widths/edges.
- [ ] Share section heading, description, body spacing and surface treatment.
- [ ] Align field labels, inputs and actions; use consistent standard control heights.
- [ ] Wrap long source paths/actions without clipping at minimum window width.
- [ ] Preserve source-setup focus, folder browsing, connection/sync, appearance and removal.
- [ ] Verify connected/disconnected Dropbox, long paths, all materials in light/dark, and minimum 800x600.
- [ ] Complete red-first layout regression tests, ordered gates and two review cycles.

## Established pattern

Reviewed libadwaita's PreferencesGroup pattern of related preference rows under a
shared title/description and optional header accessory:
https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.PreferencesGroup.html
This informs grouping and alignment. No GTK dependency or external implementation
is copied. React composition provides one small SettingsSection used by the four
existing sections. Existing material/colour tokens and Manrope typography remain
the visual language.
