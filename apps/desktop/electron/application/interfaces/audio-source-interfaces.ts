export interface PreparedTrackAudioSource {
  mimeType: string;
  url: string;
}

export interface PrepareTrackAudioSourceService {
  prepare(
    libraryId: string,
    trackId: string,
  ): Promise<PreparedTrackAudioSource>;
}

export interface AudioSourceRegistry {
  register(filePath: string, mimeType: string): Promise<string>;
}
