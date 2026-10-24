import type { TrackArtworkService } from "../track-artwork-service";
import { failure, type Result } from "../result";
import type { CommandHandler } from "./command-handler";

export class TrackArtworkCommand implements CommandHandler {
  readonly command = "get_track_artwork";
  constructor(private readonly artwork: TrackArtworkService) {}
  async handle(args: unknown): Promise<Result<string | null>> {
    const input =
      args && typeof args === "object" && "input" in args ? args.input : null;
    if (
      !input ||
      typeof input !== "object" ||
      !("libraryId" in input) ||
      typeof input.libraryId !== "string" ||
      !input.libraryId.trim() ||
      !("trackId" in input) ||
      typeof input.trackId !== "string" ||
      !input.trackId.trim()
    )
      return failure("invalidInput", "Choose a track for album artwork.");
    return this.artwork.get(input.libraryId, input.trackId);
  }
}
