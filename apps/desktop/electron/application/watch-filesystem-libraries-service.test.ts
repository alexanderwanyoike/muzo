// @vitest-environment node

import { describe, expect, it, vi } from "vitest";

import type { LibraryDto } from "../../src/types";
import type { Logger } from "./interfaces/logger-interfaces";
import type { LibraryRepository } from "./interfaces/repository-interfaces";
import type {
  FilesystemLibraryChangeHandler,
  FilesystemLibraryWatcher,
} from "./interfaces/watch-interfaces";
import { WatchFilesystemLibrariesService } from "./watch-filesystem-libraries-service";

describe("WatchFilesystemLibrariesService", () => {
  it("watches persisted filesystem libraries and scans them when they change", async () => {
    const libraries = new InMemoryLibraries([
      library("fs-1", "filesystem", "/music"),
      library("dropbox-1", "dropbox", "/dropbox"),
    ]);
    const watcher = new FakeFilesystemLibraryWatcher();
    const scanner = new FakeScanner();

    const report = await new WatchFilesystemLibrariesService(
      libraries,
      watcher,
      scanner,
      loggerSpy(),
    ).watch();

    expect(report).toEqual({ librariesWatched: 1, failures: [] });
    expect(watcher.watchedLibraries()).toEqual([
      { libraryId: "fs-1", root: "/music" },
    ]);

    await watcher.change("fs-1");

    expect(scanner.scannedLibraryIds).toEqual(["fs-1"]);
  });

  it("records watcher failures and continues", async () => {
    const libraries = new InMemoryLibraries([
      library("bad", "filesystem", "/missing"),
      library("good", "filesystem", "/music"),
    ]);
    const watcher = new FakeFilesystemLibraryWatcher({
      bad: new Error("permission denied"),
    });
    const scanner = new FakeScanner();

    const report = await new WatchFilesystemLibrariesService(
      libraries,
      watcher,
      scanner,
      loggerSpy(),
    ).watch();

    expect(report).toEqual({
      librariesWatched: 1,
      failures: [{ libraryId: "bad", message: "permission denied" }],
    });
    expect(watcher.watchedLibraries()).toEqual([
      { libraryId: "bad", root: "/missing" },
      { libraryId: "good", root: "/music" },
    ]);
  });

  it("logs scan failures when a watched library changes", async () => {
    const libraries = new InMemoryLibraries([
      library("bad", "filesystem", "/music"),
    ]);
    const watcher = new FakeFilesystemLibraryWatcher();
    const scanner = new FakeScanner({ bad: new Error("scan failed") });
    const logger = loggerSpy();

    await new WatchFilesystemLibrariesService(
      libraries,
      watcher,
      scanner,
      logger,
    ).watch();

    await watcher.change("bad");

    expect(logger.error).toHaveBeenCalledWith(
      "filesystem library reconciliation failed for bad: scan failed",
    );
  });
});

class InMemoryLibraries implements LibraryRepository {
  constructor(private readonly libraries: LibraryDto[]) {}

  async add(library: LibraryDto): Promise<void> {
    this.libraries.push(library);
  }

  async findById(libraryId: string): Promise<LibraryDto | null> {
    return this.libraries.find((library) => library.id === libraryId) ?? null;
  }

  async list(): Promise<LibraryDto[]> {
    return this.libraries;
  }
}

class FakeFilesystemLibraryWatcher implements FilesystemLibraryWatcher {
  private readonly watched = new Map<string, string>();
  private readonly handlers = new Map<string, FilesystemLibraryChangeHandler>();

  constructor(private readonly failures: Record<string, Error> = {}) {}

  async watch(
    libraryId: string,
    root: string,
    onChange: FilesystemLibraryChangeHandler,
  ): Promise<void> {
    this.watched.set(libraryId, root);
    this.handlers.set(libraryId, onChange);
    const failure = this.failures[libraryId];
    if (failure) {
      throw failure;
    }
  }

  watchedLibraries(): Array<{ libraryId: string; root: string }> {
    return Array.from(this.watched.entries()).map(([libraryId, root]) => ({
      libraryId,
      root,
    }));
  }

  async change(libraryId: string): Promise<void> {
    await this.handlers.get(libraryId)?.(libraryId);
  }
}

class FakeScanner {
  readonly scannedLibraryIds: string[] = [];

  constructor(private readonly failures: Record<string, Error> = {}) {}

  async scan(libraryId: string): Promise<{ tracksScanned: number }> {
    this.scannedLibraryIds.push(libraryId);
    const failure = this.failures[libraryId];
    if (failure) {
      throw failure;
    }
    return { tracksScanned: 0 };
  }
}

function library(
  id: string,
  kind: LibraryDto["kind"],
  location: string,
): LibraryDto {
  return {
    id,
    name: "Test",
    kind,
    location,
  };
}

function loggerSpy(): Logger {
  return {
    error: vi.fn(),
  };
}
