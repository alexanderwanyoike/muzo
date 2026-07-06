// @vitest-environment node

import { describe, expect, it, vi } from "vitest";

import { AddLibraryCommand } from "./add-library-command";

describe("AddLibraryCommand", () => {
  it("validates and persists a library", async () => {
    const dependencies = dependenciesWithId("lib-1");
    const result = await new AddLibraryCommand().handle(
      {
        input: {
          name: "Local Music",
          kind: "filesystem",
          location: "/music",
        },
      },
      dependencies,
    );

    expect(result).toEqual({
      id: "lib-1",
      name: "Local Music",
      kind: "filesystem",
      location: "/music",
    });
    expect(dependencies.libraries.add).toHaveBeenCalledWith({
      id: "lib-1",
      name: "Local Music",
      kind: "filesystem",
      location: "/music",
    });
  });

  it("rejects a missing name before writing", async () => {
    const dependencies = dependenciesWithId("lib-1");

    await expect(
      new AddLibraryCommand().handle(
        {
          input: {
            name: " ",
            kind: "filesystem",
            location: "/music",
          },
        },
        dependencies,
      ),
    ).rejects.toEqual({ kind: "emptyName" });

    expect(dependencies.libraries.add).not.toHaveBeenCalled();
  });

  it("rejects a missing location before writing", async () => {
    const dependencies = dependenciesWithId("lib-1");

    await expect(
      new AddLibraryCommand().handle(
        {
          input: {
            name: "Local Music",
            kind: "filesystem",
            location: "",
          },
        },
        dependencies,
      ),
    ).rejects.toEqual({ kind: "emptyLocation" });

    expect(dependencies.libraries.add).not.toHaveBeenCalled();
  });
});

function dependenciesWithId(id: string) {
  return {
    libraries: {
      add: vi.fn(),
      list: vi.fn(),
    },
    tracks: {
      listForLibrary: vi.fn(),
    },
    generateLibraryId: () => id,
  };
}
