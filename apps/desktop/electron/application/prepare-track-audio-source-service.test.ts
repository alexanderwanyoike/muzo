// @vitest-environment node

import { describe, expect, it, vi } from "vitest";

import type { TrackDto } from "../../src/api";
import type { AudioSourceRegistry } from "./interfaces/audio-source-interfaces";
import type { TrackReader } from "./interfaces/repository-interfaces";
import { PrepareTrackAudioSourceApplicationService } from "./prepare-track-audio-source-service";

describe("PrepareTrackAudioSourceApplicationService", () => {
  it("registers the requested track file as an audio source", async () => {
    const tracks = new FakeTrackReader([
      track({ id: "trk-1", filePath: "/music/song.mp3" }),
    ]);
    const audioSources = new FakeAudioSourceRegistry();

    await expect(
      new PrepareTrackAudioSourceApplicationService(
        tracks,
        audioSources,
      ).prepare("lib-1", "trk-1"),
    ).resolves.toEqual({
      mimeType: "audio/mpeg",
      url: "http://127.0.0.1:1234/audio/1",
    });

    expect(audioSources.register).toHaveBeenCalledWith(
      "/music/song.mp3",
      "audio/mpeg",
    );
  });

  it("rejects a missing track", async () => {
    const service = new PrepareTrackAudioSourceApplicationService(
      new FakeTrackReader([track({ id: "trk-1", filePath: "/music/song.mp3" })]),
      new FakeAudioSourceRegistry(),
    );

    await expect(service.prepare("lib-1", "missing")).rejects.toEqual({
      kind: "trackNotFound",
      message: "missing",
    });
  });

  it("rejects an unsupported audio file type", async () => {
    const service = new PrepareTrackAudioSourceApplicationService(
      new FakeTrackReader([track({ id: "trk-1", filePath: "/music/song.txt" })]),
      new FakeAudioSourceRegistry(),
    );

    await expect(service.prepare("lib-1", "trk-1")).rejects.toEqual({
      kind: "unsupportedFileType",
      message: "/music/song.txt",
    });
  });
});

class FakeTrackReader implements TrackReader {
  constructor(private readonly tracks: TrackDto[]) {}

  async listForLibrary(libraryId: string): Promise<TrackDto[]> {
    return this.tracks.filter((track) => track.libraryId === libraryId);
  }
}

class FakeAudioSourceRegistry implements AudioSourceRegistry {
  register = vi.fn(async () => "http://127.0.0.1:1234/audio/1");
}

function track(input: { id: string; filePath: string }): TrackDto {
  return {
    id: input.id,
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
    filePath: input.filePath,
  };
}
