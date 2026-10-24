import type { PlaylistDto } from "../interfaces/playlist-interfaces";
import type { PlaylistService } from "../interfaces/playlist-interfaces";
import type { CommandHandler } from "./command-handler";
import { parseInput, parseNonEmptyString, parseStringArray } from "./command-arguments";

export class ReorderPlaylistEntriesCommand implements CommandHandler {
  readonly command = "reorder_playlist_entries";

  constructor(private readonly playlistService: PlaylistService) {}

  handle(args: unknown): Promise<PlaylistDto> {
    const input = parseInput(args, "reorder_playlist_entries");
    return this.playlistService.reorderEntries(
      parseNonEmptyString(input, "playlistId", "reorder_playlist_entries"),
      parseStringArray(input, "orderedEntryIds", "reorder_playlist_entries"),
    );
  }
}
