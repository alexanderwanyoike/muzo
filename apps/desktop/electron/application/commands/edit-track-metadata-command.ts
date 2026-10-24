import type { TrackMetadataRepository } from "../interfaces/repository-interfaces";
import type { CommandHandler } from "./command-handler";
import { parseEditTrackMetadataArgs } from "./metadata-command-arguments";

export class EditTrackMetadataCommand implements CommandHandler {
  readonly command = "edit_track_metadata";

  constructor(private readonly tracks: TrackMetadataRepository) {}

  handle(args: unknown): Promise<void> {
    const input = parseEditTrackMetadataArgs(args);
    return this.tracks.updateMetadataOverride(
      input.libraryId,
      input.trackId,
      input.metadataOverride,
    );
  }
}
