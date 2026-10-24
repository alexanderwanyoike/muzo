import type { LibraryKind } from "./library-record";

export interface TrackArtworkSource {
  libraryId: string;
  trackId: string;
  path: string;
  kind: LibraryKind;
  revision: string;
}
export interface TrackArtworkSources {
  find(libraryId: string, trackId: string): Promise<TrackArtworkSource | null>;
}
export interface ArtworkPicture {
  format: string;
  data: Uint8Array;
}
export interface FileArtworkReader {
  read(path: string): Promise<ArtworkPicture | null>;
}
export interface ArtworkThumbnails {
  create(picture: ArtworkPicture): string | null;
}
