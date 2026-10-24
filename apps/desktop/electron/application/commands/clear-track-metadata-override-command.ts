import type { TrackMetadataRepository } from "../interfaces/repository-interfaces";
import type { CommandHandler } from "./command-handler";
import { parseClearTrackMetadataOverrideArgs } from "./metadata-command-arguments";

export class ClearTrackMetadataOverrideCommand implements CommandHandler {
  readonly command = "clear_track_metadata_override";

  constructor(private readonly tracks: TrackMetadataRepository) {}

  handle(args: unknown): Promise<void> {
    const input = parseClearTrackMetadataOverrideArgs(args);
    return this.tracks.clearMetadataOverride(input.libraryId, input.trackId);
  }
}
