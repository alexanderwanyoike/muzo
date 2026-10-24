import type { PrepareTrackAudioSourceService, PreparedTrackAudioSource } from "../interfaces/audio-source-interfaces";
import type { Result } from "../result";
import type { CommandHandler } from "./command-handler";
import { parseInput, parseNonEmptyString } from "./command-arguments";

export class PrepareTrackAudioSourceCommand implements CommandHandler {
  readonly command = "prepare_track_audio_source";
  constructor(private readonly service: PrepareTrackAudioSourceService) {}
  async handle(args: unknown): Promise<Result<PreparedTrackAudioSource>> {
    const input = parseInput(args, this.command);
    if (!input.ok) return input;
    const libraryId = parseNonEmptyString(input.value, "libraryId", this.command);
    if (!libraryId.ok) return libraryId;
    const trackId = parseNonEmptyString(input.value, "trackId", this.command);
    if (!trackId.ok) return trackId;
    return this.service.prepare(libraryId.value, trackId.value);
  }
}
