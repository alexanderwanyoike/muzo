import type { TrackDto } from "../../../src/api";
import type { TrackRepository } from "../interfaces/repository-interfaces";
import type { CommandHandler } from "./command-handler";

export class ListTracksCommand implements CommandHandler {
  readonly command = "list_tracks";

  constructor(private readonly tracks: TrackRepository) {}

  handle(args: unknown): Promise<TrackDto[]> {
    const { libraryId } = parseListTracksArgs(args);
    return this.tracks.listForLibrary(libraryId);
  }
}

function parseListTracksArgs(args: unknown): { libraryId: string } {
  if (!args || typeof args !== "object" || !("libraryId" in args)) {
    throw { kind: "repository", message: "missing list_tracks libraryId" };
  }

  const libraryId = (args as { libraryId: unknown }).libraryId;
  if (typeof libraryId !== "string" || libraryId.trim().length === 0) {
    throw { kind: "repository", message: "invalid list_tracks libraryId" };
  }

  return { libraryId };
}
