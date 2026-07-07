// @vitest-environment node

import { describe, expect, it } from "vitest";

import type { TrackDto } from "../../src/api";
import type { LibraryDto } from "../../src/types";
import type {
  DropboxLibraryCatalog,
  DropboxSourceFile,
} from "./interfaces/dropbox-interfaces";
import type {
  LibraryRepository,
  ScannedTrack,
  ScannedTrackRepository,
} from "./interfaces/repository-interfaces";
import { ScanDropboxLibraryService } from "./scan-dropbox-library-service";

describe("ScanDropboxLibraryService", () => {
  it("scans a Dropbox library and persists remote audio files", async () => {
    const libraries = new InMemoryLibraries([
      library("lib-1", "dropbox", "/Music"),
    ]);
    const tracks = new InMemoryTracks();
    const catalog = new FakeDropboxCatalog([
      {
        path: "/Music/Album/Song.mp3",
        size: 12_345,
        modifiedEpochSeconds: 1_700_000_000,
      },
    ]);

    const report = await new ScanDropboxLibraryService(
      libraries,
      tracks,
      catalog,
      () => "trk-1",
    ).scan({ libraryId: "lib-1", accessToken: "access-123" });

    expect(report).toEqual({ tracksScanned: 1 });
    expect(catalog.calls).toEqual([
      { accessToken: "access-123", root: "/Music" },
    ]);
    expect(tracks.stored).toEqual([
      {
        id: "trk-1",
        libraryId: "lib-1",
        title: "Song",
        artist: "Unknown Artist",
        album: null,
        trackNumber: null,
        discNumber: null,
        genre: null,
        year: null,
        durationSeconds: 0,
        filePath: "/Music/Album/Song.mp3",
        fileSize: 12_345,
        fileMtime: 1_700_000_000,
      },
    ]);
  });

  it("rejects a non-Dropbox library", async () => {
    const libraries = new InMemoryLibraries([
      library("lib-1", "filesystem", "/Music"),
    ]);
    const tracks = new InMemoryTracks();
    const catalog = new FakeDropboxCatalog([]);

    await expect(
      new ScanDropboxLibraryService(
        libraries,
        tracks,
        catalog,
        () => "trk-1",
      ).scan({ libraryId: "lib-1", accessToken: "access-123" }),
    ).rejects.toEqual({ kind: "notDropboxLibrary", libraryId: "lib-1" });
    expect(catalog.calls).toEqual([]);
    expect(tracks.stored).toEqual([]);
  });

  it("rejects a missing Dropbox library", async () => {
    const tracks = new InMemoryTracks();
    const catalog = new FakeDropboxCatalog([]);

    await expect(
      new ScanDropboxLibraryService(
        new InMemoryLibraries([]),
        tracks,
        catalog,
        () => "trk-1",
      ).scan({ libraryId: "missing", accessToken: "access-123" }),
    ).rejects.toEqual({ kind: "libraryNotFound", libraryId: "missing" });
    expect(catalog.calls).toEqual([]);
    expect(tracks.stored).toEqual([]);
  });

  it("removes tracks missing from the latest Dropbox listing", async () => {
    const libraries = new InMemoryLibraries([
      library("lib-1", "dropbox", "/Music"),
    ]);
    const tracks = new InMemoryTracks([
      scannedTrack({
        id: "old-track",
        libraryId: "lib-1",
        title: "Old Song",
        filePath: "/Music/Old Song.mp3",
        fileSize: 1,
        fileMtime: 1,
      }),
    ]);
    const catalog = new FakeDropboxCatalog([
      {
        path: "/Music/New Song.mp3",
        size: 2,
        modifiedEpochSeconds: 2,
      },
    ]);

    await new ScanDropboxLibraryService(
      libraries,
      tracks,
      catalog,
      () => "new-track",
    ).scan({ libraryId: "lib-1", accessToken: "access-123" });

    expect(tracks.stored).toEqual([
      scannedTrack({
        id: "new-track",
        libraryId: "lib-1",
        title: "New Song",
        filePath: "/Music/New Song.mp3",
        fileSize: 2,
        fileMtime: 2,
      }),
    ]);
  });
});

class InMemoryLibraries implements LibraryRepository {
  constructor(private readonly libraries: LibraryDto[]) {}

  async add(library: LibraryDto): Promise<void> {
    this.libraries.push(library);
  }

  async findById(libraryId: string): Promise<LibraryDto | null> {
    return this.libraries.find((library) => library.id === libraryId) ?? null;
  }

  async list(): Promise<LibraryDto[]> {
    return this.libraries;
  }
}

class InMemoryTracks implements ScannedTrackRepository {
  readonly stored: ScannedTrack[];

  constructor(initialTracks: ScannedTrack[] = []) {
    this.stored = [...initialTracks];
  }

  async deleteByLibraryAndPath(
    libraryId: string,
    filePath: string,
  ): Promise<void> {
    const index = this.stored.findIndex(
      (track) => track.libraryId === libraryId && track.filePath === filePath,
    );
    if (index >= 0) {
      this.stored.splice(index, 1);
    }
  }

  async listForLibrary(libraryId: string): Promise<TrackDto[]> {
    return this.stored
      .filter((track) => track.libraryId === libraryId)
      .map((track) => ({
        id: track.id,
        libraryId: track.libraryId,
        title: track.title,
        artist: track.artist,
        album: track.album,
        trackNumber: track.trackNumber,
        discNumber: track.discNumber,
        genre: track.genre,
        year: track.year,
        metadataOverridden: false,
        durationSeconds: track.durationSeconds,
        filePath: track.filePath,
      }));
  }

  async upsertScannedTrack(track: ScannedTrack): Promise<void> {
    const existingIndex = this.stored.findIndex(
      (storedTrack) =>
        storedTrack.libraryId === track.libraryId &&
        storedTrack.filePath === track.filePath,
    );
    if (existingIndex >= 0) {
      this.stored[existingIndex] = track;
      return;
    }
    this.stored.push(track);
  }
}

class FakeDropboxCatalog implements DropboxLibraryCatalog {
  readonly calls: Array<{ accessToken: string; root: string }> = [];

  constructor(private readonly files: DropboxSourceFile[]) {}

  async listAudioFiles(
    accessToken: string,
    root: string,
  ): Promise<DropboxSourceFile[]> {
    this.calls.push({ accessToken, root });
    return this.files;
  }
}

function library(
  id: string,
  kind: LibraryDto["kind"],
  location: string,
): LibraryDto {
  return {
    id,
    name: "Dropbox Music",
    kind,
    location,
  };
}

interface ScannedTrackInput {
  id: string;
  libraryId: string;
  title: string;
  filePath: string;
  fileSize?: number;
  fileMtime?: number;
}

function scannedTrack(track: ScannedTrackInput): ScannedTrack {
  return {
    artist: "Unknown Artist",
    album: null,
    trackNumber: null,
    discNumber: null,
    genre: null,
    year: null,
    durationSeconds: 0,
    fileSize: 0,
    fileMtime: 0,
    ...track,
  };
}
