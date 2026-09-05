import type { LibraryRepository } from "./interfaces/repository-interfaces";
import type { LibraryScanner } from "./scan-library-service";

export interface ReconciliationFailure {
  libraryId: string;
  message: string;
}

export interface ReconciliationReport {
  librariesReconciled: number;
  failures: ReconciliationFailure[];
}

export class ReconcileFilesystemLibrariesService {
  constructor(
    private readonly libraries: LibraryRepository,
    private readonly scanLibraryService: LibraryScanner,
  ) {}

  async reconcile(): Promise<ReconciliationReport> {
    let librariesReconciled = 0;
    const failures: ReconciliationFailure[] = [];

    for (const library of await this.libraries.list()) {
      if (library.kind !== "filesystem") {
        continue;
      }

      try {
        await this.scanLibraryService.scan(library.id);
        librariesReconciled += 1;
      } catch (error) {
        failures.push({
          libraryId: library.id,
          message: errorMessage(error),
        });
      }
    }

    return { librariesReconciled, failures };
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
