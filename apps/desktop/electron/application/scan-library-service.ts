import type {
  LibraryRepository,
  ScannedTrack,
  TrackRepository,
} from "./interfaces/repository-interfaces";
import type {
  AudioFileWalker,
  AudioMetadataReader,
} from "./interfaces/scan-interfaces";

export interface ScanReport {
  tracksScanned: number;
}

export class ScanLibraryService {
  constructor(
    private readonly libraries: LibraryRepository,
    private readonly tracks: TrackRepository,
    private readonly walker: AudioFileWalker,
    private readonly reader: AudioMetadataReader,
    private readonly generateTrackId: () => string,
  ) {}

  async scan(libraryId: string): Promise<ScanReport> {
    const library = await this.libraries.findById(libraryId);
    if (!library) {
      throw { kind: "libraryNotFound", libraryId };
    }

    const walkedFiles = await this.walker.walkAudioFiles(library.location);
    const walkedPaths = new Set(walkedFiles.map((file) => file.path));

    for (const file of walkedFiles) {
      const metadata = await this.reader.read(file.path);
      await this.tracks.upsertScannedTrack({
        id: this.generateTrackId(),
        libraryId,
        title: metadata.title,
        artist: metadata.artist,
        album: metadata.album,
        trackNumber: metadata.trackNumber,
        discNumber: metadata.discNumber,
        genre: metadata.genre,
        year: metadata.year,
        durationSeconds: metadata.durationSeconds,
        filePath: file.path,
        fileSize: file.size,
        fileMtime: file.mtime,
      } satisfies ScannedTrack);
    }

    for (const track of await this.tracks.listForLibrary(libraryId)) {
      if (!walkedPaths.has(track.filePath)) {
        await this.tracks.deleteByLibraryAndPath(libraryId, track.filePath);
      }
    }

    return { tracksScanned: walkedFiles.length };
  }
}
