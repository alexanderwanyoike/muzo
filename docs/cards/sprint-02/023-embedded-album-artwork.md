- Sprint: 02
- Type: Feature
- Status: Done
- Depends on: 022 (PR #49)

# Card 023 - Display embedded album artwork

Restore the visual identity of a desktop music collection. Show real embedded
covers in Songs, playlists, the transport and a larger listening sleeve beneath
Sources. Keep missing artwork quiet and preserve playback when artwork fails.

## Acceptance criteria

- [x] Read embedded local covers through stored library/track IDs, never renderer paths.
- [x] Read Dropbox covers with owning-account access and private temporary links.
- [x] Use bounded HTTP ranges for remote metadata rather than whole-song downloads.
- [x] Keep credentials and temporary content links out of the renderer.
- [x] Decode supported raster covers into bounded native thumbnails; reject invalid data.
- [x] Preserve detailed covers with a smaller thumbnail if necessary.
- [x] Share in-flight reads, serialize expensive work and bound cached cover count.
- [x] Key the cache by the stored file size/mtime revision and check source/account access before reuse.
- [x] Load row artwork near the viewport; load current-track artwork immediately.
- [x] Reload visible covers after a manual source refresh and ignore superseded responses.
- [x] Show a quiet fallback for missing, unsupported or unavailable artwork.
- [x] Full repository gates, visual checks and two review cycles pass (272 tests).

## Established patterns and licences

Use music-metadata's MIT-licensed parseFile/parseFromTokenizer and selectCover
pattern: https://github.com/Borewit/music-metadata#parsefromtokenizer-function
Use its documented MIT-licensed @tokenizer/http and @tokenizer/range integration
for random access without downloading skipped audio data:
https://github.com/Borewit/tokenizer-http
The adapter caps requested metadata at 8 MiB and cancels after 15 seconds. It
requires honored byte ranges and aborts outstanding work on completion/failure.
No parser or tokenizer implementation is copied. Full notices for newly bundled
HTTP/range/peek-readable dependencies are included in public/licenses.

Dropbox temporary link endpoint:
https://www.dropbox.com/developers/documentation/http/documentation#files-get_temporary_link
Native raster decoding/resizing uses Electron's existing nativeImage support:
https://www.electronjs.org/docs/latest/api/native-image
No new native image dependency or image download service is needed.

## Limits and verification

Artwork is embedded PNG/JPEG or other raster data accepted by the native decoder.
Files without usable embedded art retain the disc placeholder. Directory images
and Internet cover searches are outside this slice. Maximum input picture size is
8 MiB; thumbnails are at most 256 pixels and 256 KiB of encoded text, with a
128-pixel fallback for detailed covers. Cache capacity is 128 successful results,
including known missing art; transient failures can be retried on refresh.

Real SQLite/IPC tests parse original MP3 and FLAC fixtures. A real Electron image
decoder runs under Xvfb to verify image dimensions, invalid/SVG rejection and the
detailed-cover fallback. A local HTTP server serves an 8 MiB audio fixture and
proves the cover is read with less than 128 KiB transferred. Account-bound service
and UI tests cover credentials, deduplication, file revision changes, lazy rows,
source refresh and the listening/playlist views. Fixtures are original test data,
not user music or downloaded artwork.

Review evidence: [two review cycles](../../reviews/2026-10-24-embedded-album-artwork.md).
