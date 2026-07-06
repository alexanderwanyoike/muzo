import type { TrackDto } from "../../../src/api";
import type { LibraryDto } from "../../../src/types";

export interface LibraryRepository {
  add(library: LibraryDto): Promise<void>;
  findById(libraryId: string): Promise<LibraryDto | null>;
  list(): Promise<LibraryDto[]>;
}

export interface ScannedTrack {
  id: string;
  libraryId: string;
  title: string;
  artist: string;
  album: string | null;
  trackNumber: number | null;
  discNumber: number | null;
  genre: string | null;
  year: number | null;
  durationSeconds: number;
  filePath: string;
  fileSize: number;
  fileMtime: number;
}

export interface TrackRepository {
  deleteByLibraryAndPath(libraryId: string, filePath: string): Promise<void>;
  listForLibrary(libraryId: string): Promise<TrackDto[]>;
  upsertScannedTrack(track: ScannedTrack): Promise<void>;
}
