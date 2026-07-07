import type { LibraryDto } from "../../src/types";
import type { Logger } from "./interfaces/logger-interfaces";
import type { LibraryRepository } from "./interfaces/repository-interfaces";
import type { FilesystemLibraryWatcher } from "./interfaces/watch-interfaces";
import type { LibraryScanner } from "./reconcile-filesystem-libraries-service";

export interface WatchFilesystemLibraryFailure {
  libraryId: string;
  message: string;
}

export interface WatchFilesystemLibrariesReport {
  librariesWatched: number;
  failures: WatchFilesystemLibraryFailure[];
}

export interface AddedLibraryWatcher {
  watchLibrary(
    library: LibraryDto,
  ): Promise<WatchFilesystemLibraryFailure | null>;
}

export class WatchFilesystemLibrariesService {
  constructor(
    private readonly libraries: LibraryRepository,
    private readonly filesystemWatcher: FilesystemLibraryWatcher,
    private readonly scanLibraryService: LibraryScanner,
    private readonly logger: Logger,
  ) {}

  async watch(): Promise<WatchFilesystemLibrariesReport> {
    let librariesWatched = 0;
    const failures: WatchFilesystemLibraryFailure[] = [];

    for (const library of await this.libraries.list()) {
      const failure = await this.watchLibrary(library);
      if (failure) {
        failures.push(failure);
        continue;
      }
      if (library.kind === "filesystem") {
        librariesWatched += 1;
      }
    }

    return { librariesWatched, failures };
  }

  async watchLibrary(
    library: LibraryDto,
  ): Promise<WatchFilesystemLibraryFailure | null> {
    if (library.kind !== "filesystem") {
      return null;
    }

    try {
      await this.filesystemWatcher.watch(
        library.id,
        library.location,
        (libraryId) => this.scanChangedLibrary(libraryId),
      );
      return null;
    } catch (error) {
      return {
        libraryId: library.id,
        message: errorMessage(error),
      };
    }
  }

  private async scanChangedLibrary(libraryId: string): Promise<void> {
    try {
      await this.scanLibraryService.scan(libraryId);
    } catch (error) {
      this.logger.error(
        `filesystem library reconciliation failed for ${libraryId}: ${errorMessage(error)}`,
      );
    }
  }
}

function errorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (
    error &&
    typeof error === "object" &&
    "message" in error &&
    typeof (error as { message: unknown }).message === "string"
  ) {
    return (error as { message: string }).message;
  }
  return String(error);
}
