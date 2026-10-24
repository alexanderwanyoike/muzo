import type { PlaylistDto, PlaylistRepository, PlaylistService } from "./interfaces/playlist-interfaces";
import { failure, repositoryResult, success, type Result } from "./result";

export class PlaylistApplicationService implements PlaylistService {
  constructor(
    private readonly playlists: PlaylistRepository,
    private readonly generatePlaylistId: () => string,
    private readonly generatePlaylistEntryId: () => string,
  ) {}

  async addTrack(playlistId: string, trackId: string): Promise<Result<PlaylistDto>> {
    const found = await this.findRequiredPlaylist(playlistId);
    if (!found.ok) return found;
    const playlist = found.value;
    playlist.entries.push({ id: this.generatePlaylistEntryId(), trackId, position: playlist.entries.length });
    return this.persist(playlist);
  }

  async create(name: string): Promise<Result<PlaylistDto>> {
    if (!name.trim()) return failure("emptyName", "Enter a playlist name.");
    const playlist = { id: this.generatePlaylistId(), name, entries: [] };
    const saved = await repositoryResult(() => this.playlists.add(playlist));
    return saved.ok ? success(playlist) : saved;
  }

  list(): Promise<Result<PlaylistDto[]>> { return repositoryResult(() => this.playlists.list()); }

  async removeEntry(playlistId: string, entryId: string): Promise<Result<PlaylistDto>> {
    const found = await this.findRequiredPlaylist(playlistId);
    if (!found.ok) return found;
    const playlist = found.value;
    const previousLength = playlist.entries.length;
    playlist.entries = playlist.entries.filter((entry) => entry.id !== entryId);
    if (playlist.entries.length === previousLength) return failure("invalidEntryOrder", "The playlist entry no longer exists.");
    return this.persist(playlist);
  }

  async reorderEntries(playlistId: string, orderedEntryIds: string[]): Promise<Result<PlaylistDto>> {
    const found = await this.findRequiredPlaylist(playlistId);
    if (!found.ok) return found;
    const playlist = found.value;
    if (orderedEntryIds.length !== playlist.entries.length || new Set(orderedEntryIds).size !== orderedEntryIds.length) {
      return failure("invalidEntryOrder", "Provide every playlist entry exactly once.");
    }
    const reordered: PlaylistDto["entries"] = [];
    for (const entryId of orderedEntryIds) {
      const entry = playlist.entries.find((candidate) => candidate.id === entryId);
      if (!entry) return failure("invalidEntryOrder", "The playlist entry no longer exists.");
      reordered.push(entry);
    }
    playlist.entries = reordered;
    return this.persist(playlist);
  }

  private async findRequiredPlaylist(playlistId: string): Promise<Result<PlaylistDto>> {
    const found = await repositoryResult(() => this.playlists.findById(playlistId));
    if (!found.ok) return found;
    return found.value ? success(found.value) : failure("playlistNotFound", "The playlist no longer exists.");
  }

  private async persist(playlist: PlaylistDto): Promise<Result<PlaylistDto>> {
    const normalized = { ...playlist, entries: playlist.entries.map((entry, position) => ({ ...entry, position })) };
    const saved = await repositoryResult(() => this.playlists.save(normalized));
    return saved.ok ? success(normalized) : saved;
  }
}
