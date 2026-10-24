import type { TrackMetadataRepository } from "../interfaces/repository-interfaces";
import type { CommandHandler } from "./command-handler";
import { repositoryResult, type Result } from "../result";
import { parseClearTrackMetadataOverrideArgs } from "./metadata-command-arguments";

export class ClearTrackMetadataOverrideCommand implements CommandHandler {
  readonly command = "clear_track_metadata_override";

  constructor(private readonly tracks: TrackMetadataRepository) {}

  async handle(args: unknown): Promise<Result<void>> {
    const input = await parseClearTrackMetadataOverrideArgs(args);
    return input.ok ? repositoryResult(() => this.tracks.clearMetadataOverride(input.value.libraryId, input.value.trackId)) : input;
  }
}
