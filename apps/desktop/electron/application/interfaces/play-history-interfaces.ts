export interface PlayHistoryEntry {
  id: string;
  libraryId: string;
  trackId: string;
  playedAtUnixSeconds: number;
}

export interface PlayCount {
  trackId: string;
  playCount: number;
  lastPlayedAtUnixSeconds: number | null;
}

export interface PlayHistoryRepository {
  listPlayCounts(libraryId: string): Promise<PlayCount[]>;
  recordPlay(entry: PlayHistoryEntry): Promise<void>;
}
