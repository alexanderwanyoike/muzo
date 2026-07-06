import type { TrackDto } from "../../../src/api";
import type { LibraryDto } from "../../../src/types";

export interface LibraryRepository {
  add(library: LibraryDto): Promise<void>;
  list(): Promise<LibraryDto[]>;
}

export interface TrackRepository {
  listForLibrary(libraryId: string): Promise<TrackDto[]>;
}
