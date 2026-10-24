// @vitest-environment node

import { describe, expect, it } from "vitest";

import type {
  PlaylistDto,
  PlaylistRepository,
} from "./interfaces/playlist-interfaces";
import { PlaylistApplicationService } from "./playlist-service";

describe("PlaylistApplicationService", () => {
  it("creates a named playlist", async () => {
    const repository = new InMemoryPlaylistRepository();

    const playlist = await new PlaylistApplicationService(
      repository,
      () => "playlist-1",
      () => "entry-1",
    ).create("Road Trip");

    expect(playlist).toEqual({
      ok: true,
      value: {
        id: "playlist-1",
        name: "Road Trip",
        entries: [],
      },
    });
    if (!playlist.ok) throw new Error("Playlist creation failed");
    await expect(repository.list()).resolves.toEqual([playlist.value]);
  });

  it("rejects an empty playlist name", async () => {
    await expect(
      new PlaylistApplicationService(
        new InMemoryPlaylistRepository(),
        () => "playlist-1",
        () => "entry-1",
      ).create(" "),
    ).resolves.toMatchObject({ ok: false, error: { kind: "emptyName" } });
  });

  it("adds removes and reorders playlist entries", async () => {
    const repository = new InMemoryPlaylistRepository();
    let entryNumber = 0;
    const service = new PlaylistApplicationService(
      repository,
      () => "playlist-1",
      () => {
        entryNumber += 1;
        return `entry-${entryNumber}`;
      },
    );
    const created = await service.create("Road Trip");
    expect(created.ok).toBe(true);
    if (!created.ok) throw new Error("Playlist creation failed");
    const playlist = created.value;

    await service.addTrack(playlist.id, "track-1");
    const withTwoTracks = await service.addTrack(playlist.id, "track-2");

    expect(
      withTwoTracks.ok ? withTwoTracks.value.entries : withTwoTracks,
    ).toEqual([
      { id: "entry-1", trackId: "track-1", position: 0 },
      { id: "entry-2", trackId: "track-2", position: 1 },
    ]);

    const reordered = await service.reorderEntries(playlist.id, [
      "entry-2",
      "entry-1",
    ]);
    expect(reordered.ok ? reordered.value.entries : reordered).toEqual([
      { id: "entry-2", trackId: "track-2", position: 0 },
      { id: "entry-1", trackId: "track-1", position: 1 },
    ]);

    const removed = await service.removeEntry(playlist.id, "entry-2");
    expect(removed.ok ? removed.value.entries : removed).toEqual([
      { id: "entry-1", trackId: "track-1", position: 0 },
    ]);
  });
});

class InMemoryPlaylistRepository implements PlaylistRepository {
  private playlists: PlaylistDto[] = [];

  async add(playlist: PlaylistDto): Promise<void> {
    this.playlists.push(clonePlaylist(playlist));
  }

  async findById(playlistId: string): Promise<PlaylistDto | null> {
    const playlist = this.playlists.find((item) => item.id === playlistId);
    return playlist ? clonePlaylist(playlist) : null;
  }

  async list(): Promise<PlaylistDto[]> {
    return this.playlists.map(clonePlaylist);
  }

  async save(playlist: PlaylistDto): Promise<void> {
    const index = this.playlists.findIndex((item) => item.id === playlist.id);
    if (index >= 0) {
      this.playlists[index] = clonePlaylist(playlist);
      return;
    }
    this.playlists.push(clonePlaylist(playlist));
  }
}

function clonePlaylist(playlist: PlaylistDto): PlaylistDto {
  return {
    ...playlist,
    entries: playlist.entries.map((entry) => ({ ...entry })),
  };
}
