// @vitest-environment node

import { describe, expect, it, vi } from "vitest";

import { AddLibraryCommand } from "./add-library-command";

describe("AddLibraryCommand", () => {
  it("validates and persists a library", async () => {
    const libraries = libraryRepository();
    const watcher = addedLibraryWatcher();
    const scanner = initialScanner();
    const result = await new AddLibraryCommand(
      libraries,
      () => "lib-1",
      watcher,
      scanner,
      commandLogger(),
    ).handle({
      input: {
        name: "Local Music",
        kind: "filesystem",
        location: "/music",
      },
    });

    expect(result).toEqual({
      ok: true,
      value: {
        id: "lib-1",
        name: "Local Music",
        kind: "filesystem",
        location: "/music",
      },
    });
    expect(scanner.scan).toHaveBeenCalledWith("lib-1");
    expect(libraries.add).toHaveBeenCalledWith({
      id: "lib-1",
      name: "Local Music",
      kind: "filesystem",
      location: "/music",
    });
    expect(watcher.watchLibrary).toHaveBeenCalledWith({
      id: "lib-1",
      name: "Local Music",
      kind: "filesystem",
      location: "/music",
    });
  });

  it("rejects a missing name before writing", async () => {
    const libraries = libraryRepository();
    const watcher = addedLibraryWatcher();

    await expect(
      new AddLibraryCommand(
        libraries,
        () => "lib-1",
        watcher,
        initialScanner(),
        commandLogger(),
      ).handle({
        input: {
          name: " ",
          kind: "filesystem",
          location: "/music",
        },
      }),
    ).resolves.toMatchObject({ ok: false, error: { kind: "emptyName" } });

    expect(libraries.add).not.toHaveBeenCalled();
    expect(watcher.watchLibrary).not.toHaveBeenCalled();
  });

  it("rejects a missing location before writing", async () => {
    const libraries = libraryRepository();
    const watcher = addedLibraryWatcher();

    await expect(
      new AddLibraryCommand(
        libraries,
        () => "lib-1",
        watcher,
        initialScanner(),
        commandLogger(),
      ).handle({
        input: {
          name: "Local Music",
          kind: "filesystem",
          location: "",
        },
      }),
    ).resolves.toMatchObject({ ok: false, error: { kind: "emptyLocation" } });

    expect(libraries.add).not.toHaveBeenCalled();
    expect(watcher.watchLibrary).not.toHaveBeenCalled();
  });

  it("returns the created library when watcher registration fails", async () => {
    const libraries = libraryRepository();
    const watcher = addedLibraryWatcher();
    watcher.watchLibrary.mockResolvedValue({
      libraryId: "lib-1",
      message: "permission denied",
    });

    await expect(
      new AddLibraryCommand(
        libraries,
        () => "lib-1",
        watcher,
        initialScanner(),
        commandLogger(),
      ).handle({
        input: {
          name: "Local Music",
          kind: "filesystem",
          location: "/music",
        },
      }),
    ).resolves.toEqual({
      ok: true,
      value: {
        id: "lib-1",
        name: "Local Music",
        kind: "filesystem",
        location: "/music",
      },
    });
  });
});

function libraryRepository() {
  return {
    add: vi.fn(),
    findById: vi.fn(),
    list: vi.fn(),
  };
}

function addedLibraryWatcher() {
  return {
    watchLibrary: vi.fn().mockResolvedValue(null),
  };
}

function initialScanner() {
  return {
    scan: vi.fn(async () => ({
      ok: true as const,
      value: { tracksScanned: 0 },
    })),
  };
}
function commandLogger() {
  return { error: vi.fn() };
}
