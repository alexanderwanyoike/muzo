import type { PlayHistoryRepository } from "../interfaces/play-history-interfaces";
import type { CommandHandler } from "./command-handler";
import { repositoryResult, type Result } from "../result";
import { parseInput, parseNonEmptyString } from "./command-arguments";

export class RecordTrackPlayCommand implements CommandHandler {
  readonly command = "record_track_play";

  constructor(
    private readonly history: PlayHistoryRepository,
    private readonly generateId: () => string,
    private readonly now: () => number,
  ) {}

  async handle(args: unknown): Promise<Result<void>> {
    const input = parseInput(args, this.command);
    if (!input.ok) return input;
    const libraryId = parseNonEmptyString(
      input.value,
      "libraryId",
      this.command,
    );
    if (!libraryId.ok) return libraryId;
    const trackId = parseNonEmptyString(input.value, "trackId", this.command);
    if (!trackId.ok) return trackId;
    return repositoryResult(() =>
      this.history.recordPlay({
        id: this.generateId(),
        libraryId: libraryId.value,
        trackId: trackId.value,
        playedAtUnixSeconds: this.now(),
      }),
    );
  }
}
