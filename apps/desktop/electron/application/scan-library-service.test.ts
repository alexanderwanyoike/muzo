// @vitest-environment node

import type { LibraryDto } from "../../src/types";
import { describe, expect, it } from "vitest";
import type { TrackDto } from "../../src/api";
import type {
  LibraryRepository,
  ScannedTrack,
  ScannedTrackRepository,
} from "./interfaces/repository-interfaces";
import type {
  AudioFileWalker,
  AudioMetadata,
  AudioMetadataReader,
  WalkedAudioFile,
} from "./interfaces/scan-interfaces";
import { ScanLibraryService } from "./scan-library-service";

describe("ScanLibraryService", () => {
  it("walks the library root and persists one track per audio file", async () => {
    const libraries = new InMemoryLibraries([
      {
        id: "lib-1",
        name: "Local Music",
        kind: "filesystem",
        location: "/music",
      },
    ]);
    const tracks = new InMemoryTracks();
    const walker = new FakeWalker([
      { path: "/music/song.mp3", size: 1000, mtime: 123 },
    ]);
    const reader = new FakeMetadataReader({
      "/music/song.mp3": {
        title: "Song",
        artist: "Artist",
        album: "Album",
        trackNumber: 7,
        discNumber: 2,
        genre: "Rock",
        year: 1999,
        durationSeconds: 200,
      },
    });

    const report = await new ScanLibraryService(
      libraries,
      tracks,
      walker,
      reader,
      () => "trk-1",
    ).scan("lib-1");

    expect(report).toEqual({ tracksScanned: 1 });
    expect(walker.walkedRoots).toEqual(["/music"]);
    expect(tracks.stored).toEqual([
      {
        id: "trk-1",
        libraryId: "lib-1",
        title: "Song",
        artist: "Artist",
        album: "Album",
        trackNumber: 7,
        discNumber: 2,
        genre: "Rock",
        year: 1999,
        durationSeconds: 200,
        filePath: "/music/song.mp3",
        fileSize: 1000,
        fileMtime: 123,
      },
    ]);
  });

  it("removes tracks whose files are no longer present", async () => {
    const libraries = new InMemoryLibraries([
      {
        id: "lib-1",
        name: "Local Music",
        kind: "filesystem",
        location: "/music",
      },
    ]);
    const tracks = new InMemoryTracks([
      {
        id: "trk-old",
        libraryId: "lib-1",
        title: "Missing",
        artist: "Artist",
        album: null,
        trackNumber: null,
        discNumber: null,
        genre: null,
        year: null,
        durationSeconds: 120,
        filePath: "/music/missing.mp3",
        fileSize: 1000,
        fileMtime: 123,
      },
    ]);

    const report = await new ScanLibraryService(
      libraries,
      tracks,
      new FakeWalker([]),
      new FakeMetadataReader({}),
      () => "trk-new",
    ).scan("lib-1");

    expect(report).toEqual({ tracksScanned: 0 });
    expect(tracks.stored).toEqual([]);
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

class FakeWalker implements AudioFileWalker {
  readonly walkedRoots: string[] = [];

  constructor(private readonly files: WalkedAudioFile[]) {}

  async walkAudioFiles(root: string): Promise<WalkedAudioFile[]> {
    this.walkedRoots.push(root);
    return this.files;
  }
}

class FakeMetadataReader implements AudioMetadataReader {
  constructor(private readonly byPath: Record<string, AudioMetadata>) {}

  async read(path: string): Promise<AudioMetadata> {
    const metadata = this.byPath[path];
    if (!metadata) {
      throw new Error(`missing metadata for ${path}`);
    }
    return metadata;
  }
}
