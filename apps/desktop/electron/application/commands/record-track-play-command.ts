import type { PlayHistoryRepository } from "../interfaces/play-history-interfaces";
import type { CommandHandler } from "./command-handler";
import { parseInput, parseNonEmptyString } from "./command-arguments";

export class RecordTrackPlayCommand implements CommandHandler {
  readonly command = "record_track_play";

  constructor(
    private readonly playHistory: PlayHistoryRepository,
    private readonly generatePlayHistoryId: () => string,
    private readonly currentUnixSeconds: () => number,
  ) {}

  handle(args: unknown): Promise<void> {
    const values = parseInput(args, "record_track_play");
    const input = { libraryId: parseNonEmptyString(values, "libraryId", this.command), trackId: parseNonEmptyString(values, "trackId", this.command) };
    return this.playHistory.recordPlay({
      id: this.generatePlayHistoryId(),
      libraryId: input.libraryId,
      trackId: input.trackId,
      playedAtUnixSeconds: this.currentUnixSeconds(),
    });
  }
}
