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
  ): Promise<PreparedTrackAudioSource> {
    const tracks = await this.tracks.listForLibrary(libraryId);
    const track = tracks.find((candidate) => candidate.id === trackId);
    if (!track) {
      throw { kind: "trackNotFound", message: trackId };
    }

    const mimeType = mimeTypeForPath(track.filePath);
    if (!mimeType) {
      throw { kind: "unsupportedFileType", message: track.filePath };
    }

    return {
      mimeType,
      url: await this.audioSources.register(track.filePath, mimeType),
    };
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
