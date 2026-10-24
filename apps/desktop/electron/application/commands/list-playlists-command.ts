import type { Result } from "../result";
import type { PlaylistDto, PlaylistService  } from "../interfaces/playlist-interfaces";
import type { CommandHandler } from "./command-handler";

export class ListPlaylistsCommand implements CommandHandler {
  readonly command = "list_playlists";

  constructor(private readonly playlistService: PlaylistService) {}

  handle(): Promise<Result<PlaylistDto[]>> {
    return this.playlistService.list();
  }
}
