// @vitest-environment node
import { mkdtempSync, rmSync, copyFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { expect, it, vi } from "vitest";
import { createElectronContainer } from "./electron-container";

it("reads embedded artwork through IPC using the stored track identity", async () => {
  const directory = mkdtempSync(join(tmpdir(), "muzo-artwork-"));
  const filePath = join(directory, "song.flac");
  copyFileSync(
    fileURLToPath(
      new URL(
        "../infrastructure/metadata/fixtures/embedded-cover.flac",
        import.meta.url,
      ),
    ),
    filePath,
  );
  const thumbnail = vi.fn(
    (picture: { data: Uint8Array; format: string }) =>
      `data:${picture.format};base64,${Buffer.from(picture.data).toString("base64")}`,
  );
  const container = createElectronContainer({
    dbPath: join(directory, "catalog.sqlite"),
    artworkThumbnails: { create: thumbnail },
  });
  try {
    await container.resolve("migrateDatabase")();
    await container
      .resolve("libraries")
      .add({
        id: "source",
        name: "Music",
        kind: "filesystem",
        location: directory,
      });
    await container
      .resolve("tracks")
      .upsertScannedTrack({
        id: "track",
        libraryId: "source",
        title: "Song",
        artist: "Artist",
        album: "Album",
        trackNumber: 1,
        discNumber: 1,
        genre: null,
        year: null,
        durationSeconds: 120,
        filePath,
        fileSize: 200,
        fileMtime: 1,
      });
    const command = container.resolve("commandDispatcher");
    const result = await command.handleElectronCommand("get_track_artwork", {
      input: {
        libraryId: "source",
        trackId: "track",
        filePath: "/untrusted/ignored.flac",
      },
    });
    expect(result).toMatchObject({
      ok: true,
      value: expect.stringMatching(/^data:image\/png;base64,/),
    });
    expect(thumbnail).toHaveBeenCalledOnce();
    await expect(
      command.handleElectronCommand("get_track_artwork", {
        input: { libraryId: "other", trackId: "track" },
      }),
    ).resolves.toMatchObject({ ok: false, error: { kind: "trackNotFound" } });
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
