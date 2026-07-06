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
    const libraryRepository = {
      add: vi.fn(),
      list: vi.fn().mockResolvedValue(libraries),
    };

    await expect(
      new ListLibrariesCommand(libraryRepository).handle(),
    ).resolves.toEqual(libraries);

    expect(libraryRepository.list).toHaveBeenCalledOnce();
  });
});
