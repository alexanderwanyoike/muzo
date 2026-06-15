- Sprint: 02
- Type: Feature
- Status: Pending
- Blocks: -

# Card 007 - ID3 metadata and library editing

## What to build

Make library track metadata trustworthy and editable. Scanned tracks should
use embedded tags where available, especially ID3 tags for MP3 files, and the
library should allow the user to edit displayed track fields in Muzo.

For this card, edits are Muzo library overrides stored in the database. They
do not write back to ID3 tags yet.

## What the user can do after this card

1. Scan a folder of MP3 files.
2. See title, artist, album, track number, disc number, genre, and year from
   ID3 tags where those tags exist.
3. See sane fallbacks from the filename when tags are missing.
4. Edit track metadata in the library UI.
5. See their edits persist after app restart and rescan.
6. Keep playback working after metadata edits.

## Vertical slice

- **Domain**:
  - Extend the track model or add a metadata value object for fields Muzo
    displays and edits.
  - Make the distinction explicit between source metadata read from the file
    and user overrides stored in Muzo.
- **Application**:
  - Scanner maps embedded metadata into track metadata fields.
  - Add an edit-track-metadata use case that persists user overrides.
  - Rescan must not erase user overrides unless the user explicitly clears
    them.
- **Infrastructure**:
  - Continue using `lofty` for audio metadata reads.
  - Persist new metadata fields and override state in SQLite migrations.
- **Commands**:
  - Return enriched metadata from `list_tracks`.
  - Add a command for editing track metadata.
- **UI**:
  - Track list can show useful metadata columns, starting with title, artist,
    album, and duration.
  - Provide an edit affordance for a selected track or track row.
  - Keep editing out of the now-playing controls.

## Acceptance criteria

- [ ] MP3 scans read ID3 title, artist, album, track number, disc number,
      genre, and year when present.
- [ ] Non-MP3 formats continue to use available embedded metadata through
      `lofty` where supported.
- [ ] Missing title falls back to a clean filename stem.
- [ ] Missing artist falls back to a clear unknown-artist value.
- [ ] `TrackDto` includes album, track number, disc number, genre, year, and
      an indicator of whether displayed metadata is overridden.
- [ ] User can edit at least title, artist, album, track number, disc number,
      genre, and year for a track.
- [ ] Edits persist after app restart.
- [ ] Edits survive library rescan and automatic sync.
- [ ] User can clear an override and return to file-derived metadata.
- [ ] Editing metadata does not change the audio file or ID3 tags on disk.
- [ ] Playback continues to use the same track identity and file path after
      metadata edits.
- [ ] Database migrations are added for the new metadata and override fields.
- [ ] Unit tests cover metadata precedence: user override beats file tag,
      file tag beats filename fallback.
- [ ] Integration tests cover reading ID3 metadata from fixture MP3 files.
- [ ] UI tests cover opening the editor, saving changes, and seeing changed
      values in the track list.
- [ ] All verification commands green.

## Blocked by

- Sprint 02 card 001 - Formal SQLite migrations.
- Sprint 02 card 006 - Automatic filesystem library sync, if this card relies
  on automatic rescan preserving overrides.

## Notes for the implementer

- Do not put `serde` or Tauri types into the domain. Map metadata to DTOs at
  the command boundary.
- Use value objects for editable metadata fields where invariants matter,
  such as track number and disc number.
- Keep file-derived metadata and user overrides separate. If the same column
  tries to represent both, rescan behavior will become ambiguous.
- Prefer a selected-track details panel or modal over making every table cell
  editable in the first pass.
- The later tag-writing card should be able to reuse this editing use case,
  but this card must not write to files.

## Out of scope

- Writing ID3 tags back to disk.
- Album artwork.
- Multi-track batch editing.
- MusicBrainz or internet metadata lookup.
- Sorting and grouping by album artist or composer.
