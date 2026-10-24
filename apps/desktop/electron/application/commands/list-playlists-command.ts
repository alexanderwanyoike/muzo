import type { PlaylistDto } from "../interfaces/playlist-interfaces";
import type { PlaylistService } from "../interfaces/playlist-interfaces";
import type { CommandHandler } from "./command-handler";

export class ListPlaylistsCommand implements CommandHandler {
  readonly command = "list_playlists";

  constructor(private readonly playlistService: PlaylistService) {}

  handle(): Promise<PlaylistDto[]> {
    return this.playlistService.list();
  }
}
