// @vitest-environment node

import { describe, expect, it, vi } from "vitest";

import { handleElectronCommand } from "./commands";

describe("Electron command dispatcher", () => {
  it("routes registered commands to their handler", async () => {
    const dependencies = {
      libraries: {
        add: vi.fn(),
        list: vi.fn().mockResolvedValue([]),
      },
      tracks: {
        listForLibrary: vi.fn(),
      },
      generateLibraryId: vi.fn(),
    };

    await expect(
      handleElectronCommand("list_libraries", undefined, dependencies),
    ).resolves.toEqual([]);

    expect(dependencies.libraries.list).toHaveBeenCalledOnce();
  });

  it("rejects unregistered commands", async () => {
    await expect(handleElectronCommand("scan_library")).rejects.toThrow(
      "Electron command is not implemented: scan_library",
    );
  });
});
