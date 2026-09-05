// @vitest-environment node

import { describe, expect, it, vi } from "vitest";

import { AddLibraryCommand } from "./add-library-command";

describe("AddLibraryCommand", () => {
  it("validates and persists a library", async () => {
    const libraries = libraryRepository();
    const watcher = addedLibraryWatcher();
    const result = await new AddLibraryCommand(
      libraries,
      () => "lib-1",
      watcher,
    ).handle({
      input: {
        name: "Local Music",
        kind: "filesystem",
        location: "/music",
      },
    });

    expect(result).toEqual({
      id: "lib-1",
      name: "Local Music",
      kind: "filesystem",
      location: "/music",
    });
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
      new AddLibraryCommand(libraries, () => "lib-1", watcher).handle({
        input: {
          name: " ",
          kind: "filesystem",
          location: "/music",
        },
      }),
    ).rejects.toEqual({ kind: "emptyName" });

    expect(libraries.add).not.toHaveBeenCalled();
    expect(watcher.watchLibrary).not.toHaveBeenCalled();
  });

  it("rejects a missing location before writing", async () => {
    const libraries = libraryRepository();
    const watcher = addedLibraryWatcher();

    await expect(
      new AddLibraryCommand(libraries, () => "lib-1", watcher).handle({
        input: {
          name: "Local Music",
          kind: "filesystem",
          location: "",
        },
      }),
    ).rejects.toEqual({ kind: "emptyLocation" });

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
      new AddLibraryCommand(libraries, () => "lib-1", watcher).handle({
        input: {
          name: "Local Music",
          kind: "filesystem",
          location: "/music",
        },
      }),
    ).resolves.toEqual({
      id: "lib-1",
      name: "Local Music",
      kind: "filesystem",
      location: "/music",
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
