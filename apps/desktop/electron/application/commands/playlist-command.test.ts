// @vitest-environment node

import { describe, expect, it, vi } from "vitest";

import {
  AddTrackToPlaylistCommand,
  CreatePlaylistCommand,
  ListPlaylistsCommand,
  RemovePlaylistEntryCommand,
  ReorderPlaylistEntriesCommand,
} from "./playlist-command";

const playlist = {
  id: "playlist-1",
  name: "Road Trip",
  entries: [],
};

describe("playlist commands", () => {
  it("creates a playlist", async () => {
    const service = fakePlaylistService({
      create: vi.fn().mockResolvedValue({ ok: true, value: playlist }),
    });

    await expect(
      new CreatePlaylistCommand(service).handle({
        input: { name: "Road Trip" },
      }),
    ).resolves.toEqual({ ok: true, value: playlist });

    expect(service.create).toHaveBeenCalledWith("Road Trip");
  });

  it("lists playlists", async () => {
    const service = fakePlaylistService({
      list: vi.fn().mockResolvedValue({ ok: true, value: [playlist] }),
    });

    await expect(new ListPlaylistsCommand(service).handle()).resolves.toEqual({
      ok: true,
      value: [playlist],
    });

    expect(service.list).toHaveBeenCalledWith();
  });

  it("adds a track to a playlist", async () => {
    const service = fakePlaylistService({
      addTrack: vi.fn().mockResolvedValue({ ok: true, value: playlist }),
    });

    await expect(
      new AddTrackToPlaylistCommand(service).handle({
        input: { playlistId: "playlist-1", trackId: "track-1" },
      }),
    ).resolves.toEqual({ ok: true, value: playlist });

    expect(service.addTrack).toHaveBeenCalledWith("playlist-1", "track-1");
  });

  it("removes a playlist entry", async () => {
    const service = fakePlaylistService({
      removeEntry: vi.fn().mockResolvedValue({ ok: true, value: playlist }),
    });

    await expect(
      new RemovePlaylistEntryCommand(service).handle({
        input: { playlistId: "playlist-1", entryId: "entry-1" },
      }),
    ).resolves.toEqual({ ok: true, value: playlist });

    expect(service.removeEntry).toHaveBeenCalledWith("playlist-1", "entry-1");
  });

  it("reorders playlist entries", async () => {
    const service = fakePlaylistService({
      reorderEntries: vi.fn().mockResolvedValue({ ok: true, value: playlist }),
    });

    await expect(
      new ReorderPlaylistEntriesCommand(service).handle({
        input: {
          playlistId: "playlist-1",
          orderedEntryIds: ["entry-2", "entry-1"],
        },
      }),
    ).resolves.toEqual({ ok: true, value: playlist });

    expect(service.reorderEntries).toHaveBeenCalledWith("playlist-1", [
      "entry-2",
      "entry-1",
    ]);
  });
});

function fakePlaylistService(
  overrides: Partial<{
    addTrack: ReturnType<typeof vi.fn>;
    create: ReturnType<typeof vi.fn>;
    list: ReturnType<typeof vi.fn>;
    removeEntry: ReturnType<typeof vi.fn>;
    reorderEntries: ReturnType<typeof vi.fn>;
  }>,
) {
  return {
    addTrack: vi.fn(),
    create: vi.fn(),
    list: vi.fn(),
    removeEntry: vi.fn(),
    reorderEntries: vi.fn(),
    ...overrides,
  };
}
