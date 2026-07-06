// @vitest-environment node

import { describe, expect, it, vi } from "vitest";

import { ListLibrariesCommand } from "./list-libraries-command";

describe("ListLibrariesCommand", () => {
  it("lists libraries through the configured repository", async () => {
    const libraries = [
      {
        id: "lib-1",
        name: "Local Music",
        kind: "filesystem" as const,
        location: "/music",
      },
    ];
    const dependencies = {
      libraries: {
        add: vi.fn(),
        list: vi.fn().mockResolvedValue(libraries),
      },
      tracks: {
        listForLibrary: vi.fn(),
      },
      generateLibraryId: vi.fn(),
    };

    await expect(
      new ListLibrariesCommand().handle(undefined, dependencies),
    ).resolves.toEqual(libraries);

    expect(dependencies.libraries.list).toHaveBeenCalledOnce();
  });
});
