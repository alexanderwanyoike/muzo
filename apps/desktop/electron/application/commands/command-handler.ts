import type {
  LibraryRepository,
  TrackRepository,
} from "../interfaces/repository-interfaces";

export interface CommandDependencies {
  libraries: LibraryRepository;
  tracks: TrackRepository;
  generateLibraryId: () => string;
}

export interface CommandHandler {
  readonly command: string;
  handle(args: unknown, dependencies: CommandDependencies): Promise<unknown>;
}
