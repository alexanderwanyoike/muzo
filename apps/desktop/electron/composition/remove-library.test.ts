// @vitest-environment node
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { asValue } from "awilix";
import { expect, it, vi } from "vitest";
import { createElectronContainer } from "./electron-container";

it.each(["filesystem", "dropbox"] as const)("removes a %s source while preserving files, other sources and playlist entries", async (kind) => {
  const directory = mkdtempSync(join(tmpdir(), "muzo-remove-"));
  const filePath = join(directory, "song.mp3");
  writeFileSync(filePath, "disposable music fixture");
  const container = createElectronContainer({ dbPath: join(directory, "catalog.sqlite"), generatePlaylistId: () => "playlist", generatePlaylistEntryId: () => "entry" });
  const unwatch = vi.fn().mockResolvedValue(undefined);
  container.register({ filesystemWatcher: asValue({ watch: vi.fn(), unwatch }) });
  try {
    await container.resolve("migrateDatabase")();
    const libraries = container.resolve("libraries");
    const tracks = container.resolve("tracks");
    const dispatcher = container.resolve("commandDispatcher");
    const source = { id: "source", name: "Music", kind, location: directory };
    if (kind === "dropbox") await container.resolve("dropboxSettings").register(source, "owner");
    else await libraries.add(source);
    await libraries.add({ id: "keep", name: "Other music", kind: "filesystem", location: directory });
    await tracks.upsertScannedTrack({ id: "track", libraryId: "source", title: "Song", artist: "Artist", album: null, trackNumber: null, discNumber: null, genre: null, year: null, durationSeconds: 120, filePath, fileSize: 24, fileMtime: 1 });
    await dispatcher.handleElectronCommand("create_playlist", { input: { name: "Keep this playlist" } });
    await dispatcher.handleElectronCommand("add_track_to_playlist", { input: { playlistId: "playlist", trackId: "track" } });
    await expect(dispatcher.handleElectronCommand("remove_library", { input: { libraryId: "source" } })).resolves.toEqual({ ok: true, value: undefined });
    expect(unwatch).toHaveBeenCalledWith("source");
    expect(await container.resolve("dropboxSettings").accountFor("source")).toBeNull();
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


it("rejects malformed removal input before touching the catalogue", async () => {
  const container = createElectronContainer();
  const dispatcher = container.resolve("commandDispatcher");
  await expect(dispatcher.handleElectronCommand("remove_library", { input: { libraryId: "" } })).resolves.toMatchObject({ ok: false, error: { kind: "invalidInput" } });
});
