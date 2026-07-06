// @vitest-environment node

import { describe, expect, it, vi } from "vitest";

import { ListTracksCommand } from "./list-tracks-command";

describe("ListTracksCommand", () => {
  it("lists tracks for the requested library", async () => {
    const tracks = [
      {
        id: "trk-1",
        libraryId: "lib-1",
        title: "Song",
        artist: "Artist",
        album: null,
        trackNumber: null,
        discNumber: null,
        genre: null,
        year: null,
        metadataOverridden: false,
        durationSeconds: 120,
        filePath: "/music/song.mp3",
      },
    ];
    const dependencies = {
      libraries: {
        add: vi.fn(),
        list: vi.fn(),
      },
      tracks: {
        listForLibrary: vi.fn().mockResolvedValue(tracks),
      },
      generateLibraryId: vi.fn(),
    };

    await expect(
      new ListTracksCommand().handle({ libraryId: "lib-1" }, dependencies),
    ).resolves.toEqual(tracks);

    expect(dependencies.tracks.listForLibrary).toHaveBeenCalledWith("lib-1");
  });
});
