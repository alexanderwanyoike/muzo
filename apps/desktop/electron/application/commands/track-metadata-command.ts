import type {
  TrackMetadataOverride,
  TrackMetadataRepository,
} from "../interfaces/repository-interfaces";
import type { CommandHandler } from "./command-handler";

export class EditTrackMetadataCommand implements CommandHandler {
  readonly command = "edit_track_metadata";

  constructor(private readonly tracks: TrackMetadataRepository) {}

  handle(args: unknown): Promise<void> {
    const input = parseEditTrackMetadataArgs(args);
    return this.tracks.updateMetadataOverride(
      input.libraryId,
      input.trackId,
      input.metadataOverride,
    );
  }
}

export class ClearTrackMetadataOverrideCommand implements CommandHandler {
  readonly command = "clear_track_metadata_override";

  constructor(private readonly tracks: TrackMetadataRepository) {}

  handle(args: unknown): Promise<void> {
    const input = parseClearTrackMetadataOverrideArgs(args);
    return this.tracks.clearMetadataOverride(input.libraryId, input.trackId);
  }
}

function parseEditTrackMetadataArgs(args: unknown): {
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

function parseClearTrackMetadataOverrideArgs(args: unknown): {
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

function parseInput(
  args: unknown,
  command: string,
): Record<string, unknown> {
  if (!args || typeof args !== "object" || !("input" in args)) {
    throw { kind: "repository", message: `missing ${command} input` };
  }

  const input = (args as { input: unknown }).input;
  if (!input || typeof input !== "object") {
    throw { kind: "repository", message: `invalid ${command} input` };
  }

  return input as Record<string, unknown>;
}

function parseNonEmptyString(
  input: Record<string, unknown>,
  field: string,
  command: string,
): string {
  const value = input[field];
  if (typeof value !== "string" || value.trim().length === 0) {
    throw { kind: "repository", message: `invalid ${command} ${field}` };
  }
  return value;
}

function parseNullableString(
  input: Record<string, unknown>,
  field: string,
  command: string,
): string | null {
  const value = input[field];
  if (value === null) {
    return null;
  }
  if (typeof value !== "string") {
    throw { kind: "repository", message: `invalid ${command} ${field}` };
  }
  return value;
}

function parseNullableNumber(
  input: Record<string, unknown>,
  field: string,
  command: string,
): number | null {
  const value = input[field];
  if (value === null) {
    return null;
  }
  if (typeof value !== "number") {
    throw { kind: "repository", message: `invalid ${command} ${field}` };
  }
  return value;
}
