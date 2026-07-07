import type {
  PlaylistDto,
  PlaylistRepository,
  PlaylistService,
} from "./interfaces/playlist-interfaces";

export class PlaylistApplicationService implements PlaylistService {
  constructor(
    private readonly playlists: PlaylistRepository,
    private readonly generatePlaylistId: () => string,
    private readonly generatePlaylistEntryId: () => string,
  ) {}

  async addTrack(playlistId: string, trackId: string): Promise<PlaylistDto> {
    const playlist = await this.findRequiredPlaylist(playlistId);
    playlist.entries.push({
      id: this.generatePlaylistEntryId(),
      trackId,
      position: playlist.entries.length,
    });
    await this.playlists.save(playlist);
    return playlist;
  }

  async create(name: string): Promise<PlaylistDto> {
    if (name.trim().length === 0) {
      throw { kind: "emptyName" };
    }

    const playlist = {
      id: this.generatePlaylistId(),
      name,
      entries: [],
    };
    await this.playlists.add(playlist);
    return playlist;
  }

  list(): Promise<PlaylistDto[]> {
    return this.playlists.list();
  }

  async removeEntry(playlistId: string, entryId: string): Promise<PlaylistDto> {
    const playlist = await this.findRequiredPlaylist(playlistId);
    const previousLength = playlist.entries.length;
    playlist.entries = playlist.entries.filter((entry) => entry.id !== entryId);
    if (playlist.entries.length === previousLength) {
      throw { kind: "invalidEntryOrder" };
    }

    compactPositions(playlist);
    await this.playlists.save(playlist);
    return playlist;
  }

  async reorderEntries(
    playlistId: string,
    orderedEntryIds: string[],
  ): Promise<PlaylistDto> {
    const playlist = await this.findRequiredPlaylist(playlistId);
    if (orderedEntryIds.length !== playlist.entries.length) {
      throw { kind: "invalidEntryOrder" };
    }

    const reordered = orderedEntryIds.map((entryId) => {
      const entry = playlist.entries.find((candidate) => candidate.id === entryId);
      if (!entry) {
        throw { kind: "invalidEntryOrder" };
      }
      return entry;
    });

    if (new Set(reordered.map((entry) => entry.id)).size !== reordered.length) {
      throw { kind: "invalidEntryOrder" };
    }

    playlist.entries = reordered;
    compactPositions(playlist);
    await this.playlists.save(playlist);
    return playlist;
  }

  private async findRequiredPlaylist(playlistId: string): Promise<PlaylistDto> {
    const playlist = await this.playlists.findById(playlistId);
    if (!playlist) {
      throw { kind: "playlistNotFound" };
    }
    return playlist;
  }
}

function compactPositions(playlist: PlaylistDto): void {
  playlist.entries = playlist.entries.map((entry, position) => ({
    ...entry,
    position,
  }));
}
