import type { Result } from "../result";
import type { PlaylistDto } from "../interfaces/playlist-interfaces";
import type { PlaylistService } from "../interfaces/playlist-interfaces";
import type { CommandHandler } from "./command-handler";
import { parseInput, parseNonEmptyString } from "./command-arguments";

export class AddTrackToPlaylistCommand implements CommandHandler {
  readonly command = "add_track_to_playlist";

  constructor(private readonly playlistService: PlaylistService) {}

  handle(args: unknown): Promise<Result<PlaylistDto>> {
    const input = parseInput(args, "add_track_to_playlist");
    return this.playlistService.addTrack(
      parseNonEmptyString(input, "playlistId", "add_track_to_playlist"),
      parseNonEmptyString(input, "trackId", "add_track_to_playlist"),
    );
  }
}
