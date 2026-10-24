import type {
  PlaylistDto,
  PlaylistService,
} from "../interfaces/playlist-interfaces";
import type { CommandHandler } from "./command-handler";
import type { Result } from "../result";
import { parseInput, parseNonEmptyString } from "./command-arguments";

export class AddTrackToPlaylistCommand implements CommandHandler {
  readonly command = "add_track_to_playlist";

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
    const trackId = parseNonEmptyString(input.value, "trackId", this.command);
    if (!trackId.ok) return trackId;
    return this.service.addTrack(playlistId.value, trackId.value);
  }
}
