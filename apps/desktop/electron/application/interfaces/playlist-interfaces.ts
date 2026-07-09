export interface PlaylistEntryDto {
  id: string;
  trackId: string;
  trackArtist?: string | null;
  trackTitle?: string | null;
  position: number;
}

export interface PlaylistDto {
  id: string;
  name: string;
  entries: PlaylistEntryDto[];
}

export interface PlaylistService {
  addTrack(playlistId: string, trackId: string): Promise<PlaylistDto>;
  create(name: string): Promise<PlaylistDto>;
  list(): Promise<PlaylistDto[]>;
  removeEntry(playlistId: string, entryId: string): Promise<PlaylistDto>;
  reorderEntries(
    playlistId: string,
    orderedEntryIds: string[],
  ): Promise<PlaylistDto>;
}

export interface PlaylistRepository {
  add(playlist: PlaylistDto): Promise<void>;
  findById(playlistId: string): Promise<PlaylistDto | null>;
  list(): Promise<PlaylistDto[]>;
  save(playlist: PlaylistDto): Promise<void>;
}
