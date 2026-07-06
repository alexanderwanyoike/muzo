// @vitest-environment node

import { describe, expect, it, vi } from "vitest";

import { handleElectronCommand } from "./commands";

describe("Electron command dispatcher", () => {
  it("routes list_libraries to the configured library repository", async () => {
    const libraries = {
      list: vi.fn().mockResolvedValue([
        {
          id: "lib-1",
          name: "Local Music",
          kind: "filesystem",
          location: "/music",
        },
      ]),
    };

    await expect(
      handleElectronCommand("list_libraries", undefined, { libraries }),
    ).resolves.toEqual([
      {
        id: "lib-1",
        name: "Local Music",
        kind: "filesystem",
        location: "/music",
      },
    ]);

    expect(libraries.list).toHaveBeenCalledOnce();
  });

  it("rejects commands that have not been ported yet", async () => {
    await expect(handleElectronCommand("scan_library")).rejects.toThrow(
      "Electron command is not implemented: scan_library",
    );
  });
});
