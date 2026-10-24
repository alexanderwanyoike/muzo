export interface TrackRecord {
  id: string;
  libraryId: string;
  title: string;
  artist: string;
  album: string | null;
  trackNumber: number | null;
  discNumber: number | null;
  genre: string | null;
  year: number | null;
  metadataOverridden: boolean;
  durationSeconds: number;
  filePath: string;
}
