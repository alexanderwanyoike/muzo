// @vitest-environment node

import { describe, expect, it, vi } from "vitest";

import { ScanLibraryCommand } from "./scan-library-command";

describe("ScanLibraryCommand", () => {
  it("scans the requested library", async () => {
    const service = {
      scan: vi.fn().mockResolvedValue({ tracksScanned: 2 }),
    };

    await expect(
      new ScanLibraryCommand(service).handle({
        input: { libraryId: "lib-1" },
      }),
    ).resolves.toEqual({ tracksScanned: 2 });

    expect(service.scan).toHaveBeenCalledWith("lib-1");
  });
});
