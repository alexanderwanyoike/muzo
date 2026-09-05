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

export interface TrackMetadataOverride {
  title: string | null;
  artist: string | null;
  album: string | null;
  trackNumber: number | null;
  discNumber: number | null;
  genre: string | null;
  year: number | null;
}

export interface TrackReader {
  listForLibrary(libraryId: string): Promise<TrackDto[]>;
}

export interface ScannedTrackRepository extends TrackReader {
  deleteByLibraryAndPath(libraryId: string, filePath: string): Promise<void>;
  upsertScannedTrack(track: ScannedTrack): Promise<void>;
}

export interface TrackMetadataRepository {
  clearMetadataOverride(libraryId: string, trackId: string): Promise<void>;
  updateMetadataOverride(
    libraryId: string,
    trackId: string,
    metadataOverride: TrackMetadataOverride,
  ): Promise<void>;
}

export interface TrackRepository
  extends ScannedTrackRepository,
    TrackMetadataRepository {}
