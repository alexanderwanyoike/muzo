import type {
  PreparedTrackAudioSource,
  PrepareTrackAudioSourceService,
} from "../interfaces/audio-source-interfaces";
import type { CommandHandler } from "./command-handler";

export class PrepareTrackAudioSourceCommand implements CommandHandler {
  readonly command = "prepare_track_audio_source";

  constructor(
    private readonly prepareTrackAudioSourceService: PrepareTrackAudioSourceService,
  ) {}

  handle(args: unknown): Promise<PreparedTrackAudioSource> {
    const input = parsePrepareTrackAudioSourceArgs(args);
    return this.prepareTrackAudioSourceService.prepare(
      input.libraryId,
      input.trackId,
    );
  }
}

function parsePrepareTrackAudioSourceArgs(args: unknown): {
  libraryId: string;
  trackId: string;
} {
  if (!args || typeof args !== "object" || !("input" in args)) {
    throw {
      kind: "repository",
      message: "missing prepare_track_audio_source input",
    };
  }

  const input = (args as { input: unknown }).input;
  if (!input || typeof input !== "object") {
    throw {
      kind: "repository",
      message: "invalid prepare_track_audio_source input",
    };
  }

  return {
    libraryId: parseNonEmptyString(input, "libraryId"),
    trackId: parseNonEmptyString(input, "trackId"),
  };
}

function parseNonEmptyString(input: object, field: string): string {
  const value = (input as Record<string, unknown>)[field];
  if (typeof value !== "string" || value.trim().length === 0) {
    throw {
      kind: "repository",
      message: `invalid prepare_track_audio_source ${field}`,
    };
  }
  return value;
}
