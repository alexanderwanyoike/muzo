import type {
  PlaylistDto,
  PlaylistService,
} from "../interfaces/playlist-interfaces";
import type { CommandHandler } from "./command-handler";

export class CreatePlaylistCommand implements CommandHandler {
  readonly command = "create_playlist";

  constructor(private readonly playlistService: PlaylistService) {}

  handle(args: unknown): Promise<PlaylistDto> {
    const input = parseInput(args, "create_playlist");
    return this.playlistService.create(
      parseNonEmptyString(input, "name", "create_playlist"),
    );
  }
}

export class ListPlaylistsCommand implements CommandHandler {
  readonly command = "list_playlists";

  constructor(private readonly playlistService: PlaylistService) {}

  handle(): Promise<PlaylistDto[]> {
    return this.playlistService.list();
  }
}

export class AddTrackToPlaylistCommand implements CommandHandler {
  readonly command = "add_track_to_playlist";

  constructor(private readonly playlistService: PlaylistService) {}

  handle(args: unknown): Promise<PlaylistDto> {
    const input = parseInput(args, "add_track_to_playlist");
    return this.playlistService.addTrack(
      parseNonEmptyString(input, "playlistId", "add_track_to_playlist"),
      parseNonEmptyString(input, "trackId", "add_track_to_playlist"),
    );
  }
}

export class RemovePlaylistEntryCommand implements CommandHandler {
  readonly command = "remove_playlist_entry";

  constructor(private readonly playlistService: PlaylistService) {}

  handle(args: unknown): Promise<PlaylistDto> {
    const input = parseInput(args, "remove_playlist_entry");
    return this.playlistService.removeEntry(
      parseNonEmptyString(input, "playlistId", "remove_playlist_entry"),
      parseNonEmptyString(input, "entryId", "remove_playlist_entry"),
    );
  }
}

export class ReorderPlaylistEntriesCommand implements CommandHandler {
  readonly command = "reorder_playlist_entries";

  constructor(private readonly playlistService: PlaylistService) {}

  handle(args: unknown): Promise<PlaylistDto> {
    const input = parseInput(args, "reorder_playlist_entries");
    return this.playlistService.reorderEntries(
      parseNonEmptyString(input, "playlistId", "reorder_playlist_entries"),
      parseStringArray(input, "orderedEntryIds", "reorder_playlist_entries"),
    );
  }
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

function parseStringArray(
  input: Record<string, unknown>,
  field: string,
  command: string,
): string[] {
  const value = input[field];
  if (
    !Array.isArray(value) ||
    value.some((item) => typeof item !== "string" || item.trim().length === 0)
  ) {
    throw { kind: "repository", message: `invalid ${command} ${field}` };
  }
  return value;
}
