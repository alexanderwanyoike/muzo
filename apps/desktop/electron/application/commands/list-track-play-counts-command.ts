import type { PlayCount } from "../interfaces/play-history-interfaces";
import type { PlayHistoryRepository } from "../interfaces/play-history-interfaces";
import type { CommandHandler } from "./command-handler";
import { parseInput, parseNonEmptyString } from "./command-arguments";

export class ListTrackPlayCountsCommand implements CommandHandler {
  readonly command = "list_track_play_counts";

  constructor(private readonly playHistory: PlayHistoryRepository) {}

  handle(args: unknown): Promise<PlayCount[]> {
    const values = parseInput(args, "list_track_play_counts");
    const input = { libraryId: parseNonEmptyString(values, "libraryId", this.command) };
    return this.playHistory.listPlayCounts(input.libraryId);
  }
}
