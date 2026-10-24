import type {
  PlaylistDto,
  PlaylistService,
} from "../interfaces/playlist-interfaces";
import type { CommandHandler } from "./command-handler";
import type { Result } from "../result";
import { parseInput, parseNonEmptyString } from "./command-arguments";

export class RemovePlaylistEntryCommand implements CommandHandler {
  readonly command = "remove_playlist_entry";

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
    const entryId = parseNonEmptyString(input.value, "entryId", this.command);
    if (!entryId.ok) return entryId;
    return this.service.removeEntry(playlistId.value, entryId.value);
  }
}
