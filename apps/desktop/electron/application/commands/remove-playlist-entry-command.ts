import type { PlaylistDto } from "../interfaces/playlist-interfaces";
import type { PlaylistService } from "../interfaces/playlist-interfaces";
import type { CommandHandler } from "./command-handler";
import { parseInput, parseNonEmptyString } from "./command-arguments";

export class RemovePlaylistEntryCommand implements CommandHandler {
  readonly command = "remove_playlist_entry";

  constructor(private readonly playlistService: PlaylistService) {}

  handle(args: unknown): Promise<PlaylistDto> {
    const input = parseInput(args, "remove_playlist_entry");
    return this.playlistService.removeEntry(
      parseNonEmptyString(input, "playlistId", "remove_playlist_entry"),
      parseNonEmptyString(input, "entryId", "remove_playlist_entry"),
    );
  }
}
