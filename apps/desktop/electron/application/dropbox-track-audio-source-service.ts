import { failure, type Result } from "../../shared/result";
import type { DropboxLibraryService } from "./dropbox-library-service";
import type {
  AudioSourceRegistry,
  PreparedTrackAudioSource,
} from "./interfaces/audio-source-interfaces";
import type { DropboxAudioDownload } from "./interfaces/dropbox-audio-download";
import type { TrackReader } from "./interfaces/repository-interfaces";
import { mimeTypeForPath } from "./prepare-track-audio-source-service";

export class DropboxTrackAudioSourceService {
  constructor(
    private readonly libraries: Pick<DropboxLibraryService, "accessFor">,
    private readonly tracks: TrackReader,
    private readonly downloads: DropboxAudioDownload,
    private readonly sources: AudioSourceRegistry,
  ) {}

  async prepare(
    libraryId: string,
    trackId: string,
  ): Promise<Result<PreparedTrackAudioSource>> {
    const access = await this.libraries.accessFor(libraryId);
    if (!access.ok) return access;
    try {
      const track = (await this.tracks.listForLibrary(libraryId)).find(
        (candidate) => candidate.id === trackId,
      );
      if (!track)
        return failure(
          "invalidInput",
          "This Dropbox track is no longer in the library. Sync the library and try again.",
        );
      const mimeType = mimeTypeForPath(track.filePath);
      if (!mimeType)
        return failure("invalidInput", "This audio format is not supported.");
      const downloaded = await this.downloads.download(
        access.value.accessToken,
        `${libraryId}:${trackId}`,
        track.filePath,
      );
      if (!downloaded.ok) return downloaded;
      return {
        ok: true,
        value: {
          mimeType,
          url: await this.sources.register(downloaded.value, mimeType),
        },
      };
    } catch {
      return failure(
        "remote",
        "Could not prepare this Dropbox track for playback.",
      );
    }
  }
}
