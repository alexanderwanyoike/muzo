import type { TrackMetadataRepository } from "../interfaces/repository-interfaces";
import type { CommandHandler } from "./command-handler";
import { repositoryResult, type Result } from "../result";
import { parseEditTrackMetadataArgs } from "./metadata-command-arguments";

export class EditTrackMetadataCommand implements CommandHandler {
  readonly command = "edit_track_metadata";

  constructor(private readonly tracks: TrackMetadataRepository) {}

  async handle(args: unknown): Promise<Result<void>> {
    const input = await parseEditTrackMetadataArgs(args);
    if (!input.ok) return input;
    const { libraryId, trackId, metadataOverride } = input.value;
    return repositoryResult(() => this.tracks.updateMetadataOverride(libraryId, trackId, metadataOverride));
  }
}
