// @vitest-environment node

import { describe, expect, it, vi } from "vitest";

import { CommandDispatcher } from "./command-dispatcher";

describe("Electron command dispatcher", () => {
  it("routes registered commands to their handler", async () => {
    const handler = {
      command: "list_libraries",
      handle: vi.fn().mockResolvedValue([]),
    };
    const dispatcher = new CommandDispatcher([handler]);

    await expect(
      dispatcher.handleElectronCommand("list_libraries"),
    ).resolves.toEqual([]);

    expect(handler.handle).toHaveBeenCalledWith(undefined);
  });

  it("rejects unregistered commands", async () => {
    const dispatcher = new CommandDispatcher([]);

    await expect(dispatcher.handleElectronCommand("scan_library")).rejects.toThrow(
      "Electron command is not implemented: scan_library",
    );
  });
});
