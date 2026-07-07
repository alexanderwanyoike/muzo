import { posix } from "node:path";

import type {
  DropboxLibraryCatalog,
  DropboxSourceFile,
} from "./interfaces/dropbox-interfaces";
import type {
  LibraryRepository,
  ScannedTrack,
  ScannedTrackRepository,
} from "./interfaces/repository-interfaces";

export interface ScanDropboxLibraryInput {
  libraryId: string;
  accessToken: string;
}

export interface DropboxScanReport {
  tracksScanned: number;
}

export class ScanDropboxLibraryService {
  constructor(
    private readonly libraries: LibraryRepository,
    private readonly tracks: ScannedTrackRepository,
    private readonly dropboxCatalog: DropboxLibraryCatalog,
    private readonly generateTrackId: () => string,
  ) {}

  async scan(input: ScanDropboxLibraryInput): Promise<DropboxScanReport> {
    const library = await this.libraries.findById(input.libraryId);
    if (!library) {
      throw { kind: "libraryNotFound", libraryId: input.libraryId };
    }
    if (library.kind !== "dropbox") {
      throw { kind: "notDropboxLibrary", libraryId: input.libraryId };
    }

    const files = await this.dropboxCatalog.listAudioFiles(
      input.accessToken,
      library.location,
    );
    const remotePaths = new Set(files.map((file) => file.path));

    for (const file of files) {
      await this.tracks.upsertScannedTrack(this.trackFromFile(input.libraryId, file));
    }

    for (const track of await this.tracks.listForLibrary(input.libraryId)) {
      if (!remotePaths.has(track.filePath)) {
        await this.tracks.deleteByLibraryAndPath(input.libraryId, track.filePath);
      }
    }

    return { tracksScanned: files.length };
  }

  private trackFromFile(libraryId: string, file: DropboxSourceFile): ScannedTrack {
    return {
      id: this.generateTrackId(),
      libraryId,
      title: titleFromRemotePath(file.path),
      artist: "Unknown Artist",
      album: null,
      trackNumber: null,
      discNumber: null,
      genre: null,
      year: null,
      durationSeconds: 0,
      filePath: file.path,
      fileSize: file.size,
      fileMtime: file.modifiedEpochSeconds,
    };
  }
}

function titleFromRemotePath(path: string): string {
  return posix.parse(path).name;
}
