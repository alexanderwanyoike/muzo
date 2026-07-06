// @vitest-environment node

import { describe, expect, it, vi } from "vitest";

import { AddLibraryCommand } from "./add-library-command";

describe("AddLibraryCommand", () => {
  it("validates and persists a library", async () => {
    const libraries = libraryRepository();
    const result = await new AddLibraryCommand(libraries, () => "lib-1").handle({
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
  });

  it("rejects a missing name before writing", async () => {
    const libraries = libraryRepository();

    await expect(
      new AddLibraryCommand(libraries, () => "lib-1").handle({
        input: {
          name: " ",
          kind: "filesystem",
          location: "/music",
        },
      }),
    ).rejects.toEqual({ kind: "emptyName" });

    expect(libraries.add).not.toHaveBeenCalled();
  });

  it("rejects a missing location before writing", async () => {
    const libraries = libraryRepository();

    await expect(
      new AddLibraryCommand(libraries, () => "lib-1").handle({
        input: {
          name: "Local Music",
          kind: "filesystem",
          location: "",
        },
      }),
    ).rejects.toEqual({ kind: "emptyLocation" });

    expect(libraries.add).not.toHaveBeenCalled();
  });
});

function libraryRepository() {
  return {
    add: vi.fn(),
    list: vi.fn(),
  };
}
