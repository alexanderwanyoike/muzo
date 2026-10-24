import type { TrackMetadataOverride } from "../interfaces/repository-interfaces";
import { success, type Result } from "../result";
import { parseInput, parseNonEmptyString, parseNullableString, parseNullableNumber } from "./command-arguments";

export async function parseEditTrackMetadataArgs(args: unknown): Promise<Result<{ libraryId: string; trackId: string; metadataOverride: TrackMetadataOverride }>> {
  const ids = await parseClearTrackMetadataOverrideArgs(args);
  if (!ids.ok) return ids;
  const input = parseInput(args, "edit_track_metadata");
  if (!input.ok) return input;
  const metadataOverride: TrackMetadataOverride = { title: null, artist: null, album: null, trackNumber: null, discNumber: null, genre: null, year: null };
  for (const field of ["title", "artist", "album", "genre"] as const) {
    const value = parseNullableString(input.value, field, "edit_track_metadata");
    if (!value.ok) return value;
    metadataOverride[field] = value.value;
  }
  for (const field of ["trackNumber", "discNumber", "year"] as const) {
    const value = parseNullableNumber(input.value, field, "edit_track_metadata");
    if (!value.ok) return value;
    metadataOverride[field] = value.value;
  }
  return success({ ...ids.value, metadataOverride });
}

export async function parseClearTrackMetadataOverrideArgs(args: unknown): Promise<Result<{ libraryId: string; trackId: string }>> {
  const input = parseInput(args, "clear_track_metadata_override");
  if (!input.ok) return input;
  const libraryId = parseNonEmptyString(input.value, "libraryId", "clear_track_metadata_override");
  if (!libraryId.ok) return libraryId;
  const trackId = parseNonEmptyString(input.value, "trackId", "clear_track_metadata_override");
  return trackId.ok ? success({ libraryId: libraryId.value, trackId: trackId.value }) : trackId;
}
