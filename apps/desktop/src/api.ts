import { invokeCommand } from "./runtime-bridge";
import type {
  AddLibraryInputDto,
  LibraryDto,
  LibraryKindDto,
} from "./types";

export async function addLibrary(input: {
  name: string;
  kind: LibraryKindDto;
  location: string;
}): Promise<LibraryDto> {
  const dto: AddLibraryInputDto = {
    name: input.name,
    kind: input.kind,
    location: input.location,
  };
  return invokeCommand<LibraryDto>("add_library", { input: dto });
}

export async function listLibraries(): Promise<LibraryDto[]> {
  return invokeCommand<LibraryDto[]>("list_libraries");
}

export interface ScanReport {
  tracksScanned: number;
}

export async function scanLibrary(libraryId: string): Promise<ScanReport> {
  return invokeCommand<ScanReport>("scan_library", {
    input: { libraryId },
  });
}

export interface TrackDto {
  id: string;
  libraryId: string;
  title: string;
  artist: string;
  album: string | null;
  trackNumber: number | null;
  discNumber: number | null;
  genre: string | null;
  year: number | null;
  metadataOverridden: boolean;
  durationSeconds: number;
  filePath: string;
}

export async function listTracks(libraryId: string): Promise<TrackDto[]> {
  return invokeCommand<TrackDto[]>("list_tracks", { libraryId });
}

export interface EditTrackMetadataInput {
  libraryId: string;
  trackId: string;
  title: string | null;
  artist: string | null;
  album: string | null;
  trackNumber: number | null;
  discNumber: number | null;
  genre: string | null;
  year: number | null;
}

export async function editTrackMetadata(
  input: EditTrackMetadataInput,
): Promise<void> {
  return invokeCommand<void>("edit_track_metadata", { input });
}

export async function clearTrackMetadataOverride(input: {
  libraryId: string;
  trackId: string;
}): Promise<void> {
  return invokeCommand<void>("clear_track_metadata_override", { input });
}

export interface PreparedTrackAudioSourceDto {
  mimeType: string;
  url: string;
}

export async function prepareTrackAudioSource(input: {
  libraryId: string;
  trackId: string;
}): Promise<PreparedTrackAudioSourceDto> {
  return invokeCommand<PreparedTrackAudioSourceDto>("prepare_track_audio_source", { input });
}

export async function recordTrackPlay(input: {
  libraryId: string;
  trackId: string;
}): Promise<void> {
  return invokeCommand<void>("record_track_play", { input });
}

export interface PlayCountDto {
  trackId: string;
  playCount: number;
  lastPlayedAtUnixSeconds: number | null;
}

export async function listTrackPlayCounts(libraryId: string): Promise<PlayCountDto[]> {
  return invokeCommand<PlayCountDto[]>("list_track_play_counts", {
    input: { libraryId },
  });
}

export interface PlaylistEntryDto {
  id: string;
  trackId: string;
  position: number;
}

export interface PlaylistDto {
  id: string;
  name: string;
  entries: PlaylistEntryDto[];
}

export async function listPlaylists(): Promise<PlaylistDto[]> {
  return invokeCommand<PlaylistDto[]>("list_playlists");
}

export async function createPlaylist(input: {
  name: string;
}): Promise<PlaylistDto> {
  return invokeCommand<PlaylistDto>("create_playlist", { input });
}

export async function addTrackToPlaylist(input: {
  playlistId: string;
  trackId: string;
}): Promise<PlaylistDto> {
  return invokeCommand<PlaylistDto>("add_track_to_playlist", { input });
}
