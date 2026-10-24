import { failure, success, type Result } from "./result";
import type {
  AudioSourceRegistry,
  PreparedTrackAudioSource,
  PrepareTrackAudioSourceService,
} from "./interfaces/audio-source-interfaces";
import type { TrackReader } from "./interfaces/repository-interfaces";

export class PrepareTrackAudioSourceApplicationService
  implements PrepareTrackAudioSourceService
{
  constructor(
    private readonly tracks: TrackReader,
    private readonly audioSources: AudioSourceRegistry,
  ) {}

  async prepare(
    libraryId: string,
    trackId: string,
  ): Promise<Result<PreparedTrackAudioSource>> {
    try {
    const tracks = await this.tracks.listForLibrary(libraryId);
    const track = tracks.find((candidate) => candidate.id === trackId);
    if (!track) {
      return failure("trackNotFound", trackId);
    }

    const mimeType = mimeTypeForPath(track.filePath);
    if (!mimeType) {
      return failure("unsupportedFileType", track.filePath);
    }

    return success({
      mimeType,
      url: await this.audioSources.register(track.filePath, mimeType),
    });
    } catch { return failure("repository", "Could not prepare the track audio source."); }
  }
}

function mimeTypeForPath(filePath: string): string | null {
  const extension = filePath.split(".").at(-1)?.toLowerCase();
  switch (extension) {
    case "mp3":
      return "audio/mpeg";
    case "flac":
      return "audio/flac";
    case "wav":
      return "audio/wav";
    case "ogg":
      return "audio/ogg";
    case "aac":
    case "m4a":
      return "audio/mp4";
    default:
      return null;
  }
}
