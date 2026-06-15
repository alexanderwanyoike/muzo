import { invoke } from "@tauri-apps/api/core";
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
  return invoke<LibraryDto>("add_library", { input: dto });
}

export async function listLibraries(): Promise<LibraryDto[]> {
  return invoke<LibraryDto[]>("list_libraries");
}

export interface ScanReport {
  tracksScanned: number;
}

export async function scanLibrary(libraryId: string): Promise<ScanReport> {
  return invoke<ScanReport>("scan_library", {
    input: { libraryId },
  });
}

export interface TrackDto {
  id: string;
  libraryId: string;
  title: string;
  artist: string;
  durationSeconds: number;
  filePath: string;
}

export async function listTracks(libraryId: string): Promise<TrackDto[]> {
  return invoke<TrackDto[]>("list_tracks", { libraryId });
}

export interface TrackAudioSourceDto {
  mimeType: string;
  bytes: number[];
}

export async function loadTrackAudioSource(input: {
  libraryId: string;
  trackId: string;
}): Promise<TrackAudioSourceDto> {
  return invoke<TrackAudioSourceDto>("load_track_audio_source", { input });
}
