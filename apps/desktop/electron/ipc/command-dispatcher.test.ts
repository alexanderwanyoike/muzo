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

    await expect(
      dispatcher.handleElectronCommand("prepare_track_audio_source"),
    ).rejects.toThrow(
      "Electron command is not implemented: prepare_track_audio_source",
    );
  });
});
