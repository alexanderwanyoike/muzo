import type { TrackMetadataOverride } from "../interfaces/repository-interfaces";
import { parseInput, parseNonEmptyString, parseNullableString, parseNullableNumber } from "./command-arguments";

export function parseEditTrackMetadataArgs(args: unknown): {
  libraryId: string;
  trackId: string;
  metadataOverride: TrackMetadataOverride;
} {
  const input = parseInput(args, "edit_track_metadata");
  const libraryId = parseNonEmptyString(input, "libraryId", "edit_track_metadata");
  const trackId = parseNonEmptyString(input, "trackId", "edit_track_metadata");

  return {
    libraryId,
    trackId,
    metadataOverride: {
      title: parseNullableString(input, "title", "edit_track_metadata"),
      artist: parseNullableString(input, "artist", "edit_track_metadata"),
      album: parseNullableString(input, "album", "edit_track_metadata"),
      trackNumber: parseNullableNumber(input, "trackNumber", "edit_track_metadata"),
      discNumber: parseNullableNumber(input, "discNumber", "edit_track_metadata"),
      genre: parseNullableString(input, "genre", "edit_track_metadata"),
      year: parseNullableNumber(input, "year", "edit_track_metadata"),
    },
  };
}

export function parseClearTrackMetadataOverrideArgs(args: unknown): {
  libraryId: string;
  trackId: string;
} {
  const input = parseInput(args, "clear_track_metadata_override");
  return {
    libraryId: parseNonEmptyString(
      input,
      "libraryId",
      "clear_track_metadata_override",
    ),
    trackId: parseNonEmptyString(input, "trackId", "clear_track_metadata_override"),
  };
}

