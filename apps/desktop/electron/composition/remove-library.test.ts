// @vitest-environment node
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, it } from "vitest";
import { createElectronContainer } from "./electron-container";

it("removes one source and its catalogue while preserving files, other sources and playlist entries", async () => {
  const directory = mkdtempSync(join(tmpdir(), "muzo-remove-"));
  const filePath = join(directory, "song.mp3");
  writeFileSync(filePath, "disposable music fixture");
  const container = createElectronContainer({ dbPath: join(directory, "catalog.sqlite"), generatePlaylistId: () => "playlist", generatePlaylistEntryId: () => "entry" });
  try {
    await container.resolve("migrateDatabase")();
    const libraries = container.resolve("libraries");
    const tracks = container.resolve("tracks");
    const dispatcher = container.resolve("commandDispatcher");
    await libraries.add({ id: "source", name: "Music", kind: "filesystem", location: directory });
    await libraries.add({ id: "keep", name: "Other music", kind: "filesystem", location: directory });
    await tracks.upsertScannedTrack({ id: "track", libraryId: "source", title: "Song", artist: "Artist", album: null, trackNumber: null, discNumber: null, genre: null, year: null, durationSeconds: 120, filePath, fileSize: 24, fileMtime: 1 });
    await dispatcher.handleElectronCommand("create_playlist", { input: { name: "Keep this playlist" } });
    await dispatcher.handleElectronCommand("add_track_to_playlist", { input: { playlistId: "playlist", trackId: "track" } });
    await expect(dispatcher.handleElectronCommand("remove_library", { input: { libraryId: "source" } })).resolves.toEqual({ ok: true, value: undefined });
    expect((await libraries.list()).map(item => item.id)).toEqual(["keep"]);
    expect(await tracks.listForLibrary("source")).toEqual([]);
    await tracks.upsertScannedTrack({ id: "late-track", libraryId: "source", title: "Late scan", artist: "Artist", album: null, trackNumber: null, discNumber: null, genre: null, year: null, durationSeconds: 120, filePath, fileSize: 24, fileMtime: 1 });
    expect(await tracks.listForLibrary("source")).toEqual([]);
    expect(readFileSync(filePath, "utf8")).toBe("disposable music fixture");
    const playlists = await container.resolve("playlists").list();
    expect(playlists[0].entries).toHaveLength(1);
    expect(playlists[0].entries[0].trackId).toBe("track");
    await expect(dispatcher.handleElectronCommand("remove_library", { input: { libraryId: "source" } })).resolves.toMatchObject({ ok: false, error: { kind: "libraryNotFound" } });
  } finally { rmSync(directory, { recursive: true, force: true }); }
});
