import type {
  PlaylistDto,
  PlaylistService,
} from "../interfaces/playlist-interfaces";
import type { CommandHandler } from "./command-handler";
import type { Result } from "../result";
import {
  parseInput,
  parseNonEmptyString,
  parseStringArray,
} from "./command-arguments";

export class ReorderPlaylistEntriesCommand implements CommandHandler {
  readonly command = "reorder_playlist_entries";

  constructor(private readonly service: PlaylistService) {}

  async handle(args: unknown): Promise<Result<PlaylistDto>> {
    const input = parseInput(args, this.command);
    if (!input.ok) return input;
    const playlistId = parseNonEmptyString(
      input.value,
      "playlistId",
      this.command,
    );
    if (!playlistId.ok) return playlistId;
    const orderedEntryIds = parseStringArray(
      input.value,
      "orderedEntryIds",
      this.command,
    );
    if (!orderedEntryIds.ok) return orderedEntryIds;
    return this.service.reorderEntries(playlistId.value, orderedEntryIds.value);
  }
}
