import type {
  PrepareTrackAudioSourceService,
  PreparedTrackAudioSource,
} from "../interfaces/audio-source-interfaces";
import { repositoryResult, type Result } from "../result";
import type { CommandHandler } from "./command-handler";
import { parseInput, parseNonEmptyString } from "./command-arguments";
import type { LibraryRepository } from "../interfaces/repository-interfaces";

export class PrepareTrackAudioSourceCommand implements CommandHandler {
  readonly command = "prepare_track_audio_source";

  constructor(
    private readonly service: PrepareTrackAudioSourceService,
    private readonly libraries?: LibraryRepository,
    private readonly dropboxSources?: PrepareTrackAudioSourceService,
  ) {}

  async handle(args: unknown): Promise<Result<PreparedTrackAudioSource>> {
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
    if (this.dropboxSources && this.libraries) {
      const library = await repositoryResult(() =>
        this.libraries!.findById(libraryId.value),
      );
      if (!library.ok) return library;
      if (library.value?.kind === "dropbox")
        return this.dropboxSources.prepare(libraryId.value, trackId.value);
    }
    return this.service.prepare(libraryId.value, trackId.value);
  }
}
