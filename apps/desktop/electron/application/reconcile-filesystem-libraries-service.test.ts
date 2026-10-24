// @vitest-environment node

import { describe, expect, it } from "vitest";
import type { Result } from "./result";

import type { LibraryDto } from "../../src/types";
import type { LibraryRepository } from "./interfaces/repository-interfaces";
import { ReconcileFilesystemLibrariesService } from "./reconcile-filesystem-libraries-service";

describe("ReconcileFilesystemLibrariesService", () => {
  it("scans persisted filesystem libraries", async () => {
    const libraries = new InMemoryLibraries([
      createLibrary("fs-1", "filesystem", "/music"),
      createLibrary("dropbox-1", "dropbox", "/dropbox"),
    ]);
    const scanner = new FakeScanner();

    const report = await new ReconcileFilesystemLibrariesService(
      libraries,
      scanner,
    ).reconcile();

    expect(report).toEqual({
      librariesReconciled: 1,
      failures: [],
    });
    expect(scanner.scannedLibraryIds).toEqual(["fs-1"]);
  });

  it("records filesystem scan failures and continues", async () => {
    const libraries = new InMemoryLibraries([
      createLibrary("bad", "filesystem", "/missing"),
      createLibrary("good", "filesystem", "/music"),
    ]);
    const scanner = new FakeScanner({ bad: new Error("not found") });

    const report = await new ReconcileFilesystemLibrariesService(
      libraries,
      scanner,
    ).reconcile();

    expect(report).toEqual({
      librariesReconciled: 1,
      failures: [{ libraryId: "bad", message: "not found" }],
    });
    expect(scanner.scannedLibraryIds).toEqual(["bad", "good"]);
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

class FakeScanner {
  readonly scannedLibraryIds: string[] = [];

  constructor(private readonly failures: Record<string, Error> = {}) {}

  async scan(libraryId: string): Promise<Result<{ tracksScanned: number }>> {
    this.scannedLibraryIds.push(libraryId);
    const failure = this.failures[libraryId];
    if (failure) {
      return {
        ok: false,
        error: { kind: "repository", message: failure.message },
      };
    }
    return { ok: true, value: { tracksScanned: 0 } };
  }
}

function createLibrary(
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
