import type { PlaylistDto } from "../interfaces/playlist-interfaces";
import type { PlaylistService } from "../interfaces/playlist-interfaces";
import type { CommandHandler } from "./command-handler";
import { parseInput, parseNonEmptyString } from "./command-arguments";

export class CreatePlaylistCommand implements CommandHandler {
  readonly command = "create_playlist";

  constructor(private readonly playlistService: PlaylistService) {}

  handle(args: unknown): Promise<PlaylistDto> {
    const input = parseInput(args, "create_playlist");
    return this.playlistService.create(
      parseNonEmptyString(input, "name", "create_playlist"),
    );
  }
}
