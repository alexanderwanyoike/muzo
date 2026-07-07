// @vitest-environment node

import { describe, expect, it, vi } from "vitest";

import {
  ClearTrackMetadataOverrideCommand,
  EditTrackMetadataCommand,
} from "./track-metadata-command";

describe("EditTrackMetadataCommand", () => {
  it("persists metadata overrides for the requested track", async () => {
    const tracks = {
      updateMetadataOverride: vi.fn().mockResolvedValue(undefined),
    };

    await expect(
      new EditTrackMetadataCommand(tracks).handle({
        input: {
          libraryId: "lib-1",
          trackId: "trk-1",
          title: "Edited Title",
          artist: null,
          album: "Edited Album",
          trackNumber: 4,
          discNumber: null,
          genre: "Soul",
          year: 1971,
        },
      }),
    ).resolves.toBeUndefined();

    expect(tracks.updateMetadataOverride).toHaveBeenCalledWith("lib-1", "trk-1", {
      title: "Edited Title",
      artist: null,
      album: "Edited Album",
      trackNumber: 4,
      discNumber: null,
      genre: "Soul",
      year: 1971,
    });
  });
});

describe("ClearTrackMetadataOverrideCommand", () => {
  it("clears metadata overrides for the requested track", async () => {
    const tracks = {
      clearMetadataOverride: vi.fn().mockResolvedValue(undefined),
    };

    await expect(
      new ClearTrackMetadataOverrideCommand(tracks).handle({
        input: { libraryId: "lib-1", trackId: "trk-1" },
      }),
    ).resolves.toBeUndefined();

    expect(tracks.clearMetadataOverride).toHaveBeenCalledWith("lib-1", "trk-1");
  });
});
