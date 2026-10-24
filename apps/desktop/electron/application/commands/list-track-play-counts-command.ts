import type { PlayCount, PlayHistoryRepository } from "../interfaces/play-history-interfaces";
import type { CommandHandler } from "./command-handler";
import { repositoryResult, type Result } from "../result";
import { parseInput, parseNonEmptyString } from "./command-arguments";

export class ListTrackPlayCountsCommand implements CommandHandler {
  readonly command = "list_track_play_counts";

  constructor(private readonly history: PlayHistoryRepository) {}

  async handle(args: unknown): Promise<Result<PlayCount[]>> {
    const input = parseInput(args, this.command);
    if (!input.ok) return input;
    const id = parseNonEmptyString(input.value, "libraryId", this.command);
    return id.ok ? repositoryResult(() => this.history.listPlayCounts(id.value)) : id;
  }
}
