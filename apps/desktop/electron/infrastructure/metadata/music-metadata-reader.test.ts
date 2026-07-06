// @vitest-environment node

import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("music-metadata", () => ({
  parseFile: vi.fn(),
}));

import { parseFile } from "music-metadata";

import { MusicMetadataReader } from "./music-metadata-reader";

const mockedParseFile = vi.mocked(parseFile);

describe("MusicMetadataReader", () => {
  beforeEach(() => {
    mockedParseFile.mockReset();
  });

  it("maps embedded audio metadata", async () => {
    mockedParseFile.mockResolvedValue({
      common: {
        title: "Song",
        artist: "Artist",
        album: "Album",
        track: { no: 7, of: 12 },
        disk: { no: 2, of: 2 },
        genre: ["Rock"],
        year: 1999,
      },
      format: {
        duration: 200.4,
      },
    } as Awaited<ReturnType<typeof parseFile>>);

    await expect(
      new MusicMetadataReader().read("/music/song.mp3"),
    ).resolves.toEqual({
      title: "Song",
      artist: "Artist",
      album: "Album",
      trackNumber: 7,
      discNumber: 2,
      genre: "Rock",
      year: 1999,
      durationSeconds: 200,
    });
  });

  it("falls back when optional metadata is missing", async () => {
    mockedParseFile.mockResolvedValue({
      common: {},
      format: {},
    } as Awaited<ReturnType<typeof parseFile>>);

    await expect(
      new MusicMetadataReader().read("/music/file-name.mp3"),
    ).resolves.toEqual({
      title: "file-name",
      artist: "Unknown Artist",
      album: null,
      trackNumber: null,
      discNumber: null,
      genre: null,
      year: null,
      durationSeconds: 0,
    });
  });
});
