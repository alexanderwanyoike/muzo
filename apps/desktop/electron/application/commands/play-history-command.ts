import type {
  PlayCount,
  PlayHistoryRepository,
} from "../interfaces/play-history-interfaces";
import type { CommandHandler } from "./command-handler";

export class RecordTrackPlayCommand implements CommandHandler {
  readonly command = "record_track_play";

  constructor(
    private readonly playHistory: Pick<PlayHistoryRepository, "recordPlay">,
    private readonly generatePlayHistoryId: () => string,
    private readonly currentUnixSeconds: () => number,
  ) {}

  handle(args: unknown): Promise<void> {
    const input = parseTrackPlayInput(args, "record_track_play");
    return this.playHistory.recordPlay({
      id: this.generatePlayHistoryId(),
      libraryId: input.libraryId,
      trackId: input.trackId,
      playedAtUnixSeconds: this.currentUnixSeconds(),
    });
  }
}

export class ListTrackPlayCountsCommand implements CommandHandler {
  readonly command = "list_track_play_counts";

  constructor(
    private readonly playHistory: Pick<PlayHistoryRepository, "listPlayCounts">,
  ) {}

  handle(args: unknown): Promise<PlayCount[]> {
    const input = parseLibraryInput(args, "list_track_play_counts");
    return this.playHistory.listPlayCounts(input.libraryId);
  }
}

function parseLibraryInput(
  args: unknown,
  command: string,
): { libraryId: string } {
  const input = parseInput(args, command);
  return {
    libraryId: parseNonEmptyString(input, "libraryId", command),
  };
}

function parseTrackPlayInput(
  args: unknown,
  command: string,
): { libraryId: string; trackId: string } {
  const input = parseInput(args, command);
  return {
    libraryId: parseNonEmptyString(input, "libraryId", command),
    trackId: parseNonEmptyString(input, "trackId", command),
  };
}

function parseInput(
  args: unknown,
  command: string,
): Record<string, unknown> {
  if (!args || typeof args !== "object" || !("input" in args)) {
    throw { kind: "repository", message: `missing ${command} input` };
  }

  const input = (args as { input: unknown }).input;
  if (!input || typeof input !== "object") {
    throw { kind: "repository", message: `invalid ${command} input` };
  }

  return input as Record<string, unknown>;
}

function parseNonEmptyString(
  input: Record<string, unknown>,
  field: string,
  command: string,
): string {
  const value = input[field];
  if (typeof value !== "string" || value.trim().length === 0) {
    throw { kind: "repository", message: `invalid ${command} ${field}` };
  }
  return value;
}
