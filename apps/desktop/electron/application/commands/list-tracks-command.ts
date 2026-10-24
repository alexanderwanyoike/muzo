import type { TrackRecord } from "../interfaces/track-record";
import type { TrackReader } from "../interfaces/repository-interfaces";
import type { CommandHandler } from "./command-handler";
import { failure, repositoryResult, type Result } from "../result";
import { parseNonEmptyString } from "./command-arguments";

export class ListTracksCommand implements CommandHandler {
  readonly command = "list_tracks";

  constructor(private readonly tracks: TrackReader) {}

  async handle(args: unknown): Promise<Result<TrackRecord[]>> {
    if (!args || typeof args !== "object" || Array.isArray(args)) return failure("repository", "missing list_tracks libraryId");
    const id = parseNonEmptyString(args as Record<string, unknown>, "libraryId", this.command);
    return id.ok ? repositoryResult(() => this.tracks.listForLibrary(id.value)) : id;
  }
}
