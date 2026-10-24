import type { Result } from "../result";
export interface PlaylistEntryDto {
  id: string;
  trackId: string;
  trackArtist?: string | null;
  trackDurationSeconds?: number | null;
  trackTitle?: string | null;
  trackLibraryId?: string | null;
  position: number;
}

export interface PlaylistDto {
  id: string;
  name: string;
  entries: PlaylistEntryDto[];
}

export interface PlaylistService {
  addTrack(playlistId: string, trackId: string): Promise<Result<PlaylistDto>>;
  create(name: string): Promise<Result<PlaylistDto>>;
  list(): Promise<Result<PlaylistDto[]>>;
  removeEntry(playlistId: string, entryId: string): Promise<Result<PlaylistDto>>;
  reorderEntries(
    playlistId: string,
    orderedEntryIds: string[],
  ): Promise<Result<PlaylistDto>>;
}

export interface PlaylistRepository {
  add(playlist: PlaylistDto): Promise<void>;
  findById(playlistId: string): Promise<PlaylistDto | null>;
  list(): Promise<PlaylistDto[]>;
  save(playlist: PlaylistDto): Promise<void>;
}
