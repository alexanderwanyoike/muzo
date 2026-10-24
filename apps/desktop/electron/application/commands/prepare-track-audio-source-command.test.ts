// @vitest-environment node

import { describe, expect, it, vi } from "vitest";

import { PrepareTrackAudioSourceCommand } from "./prepare-track-audio-source-command";

describe("PrepareTrackAudioSourceCommand", () => {
  it("prepares an audio source for the requested track", async () => {
    const service = {
      prepare: vi.fn().mockResolvedValue({ ok: true, value: ({
        mimeType: "audio/mpeg",
        url: "http://127.0.0.1:1234/audio/token",
      }) }),
    };

    await expect(
      new PrepareTrackAudioSourceCommand(service).handle({
        input: { libraryId: "lib-1", trackId: "trk-1" },
      }),
    ).resolves.toEqual({ ok: true, value: ({
      mimeType: "audio/mpeg",
      url: "http://127.0.0.1:1234/audio/token",
    }) });

    expect(service.prepare).toHaveBeenCalledWith("lib-1", "trk-1");
  });
});

it("returns a storage failure when the library kind cannot be loaded", async () => {
  const prepare = vi.fn();
  const command = new PrepareTrackAudioSourceCommand({ prepare }, {
    add: async () => {}, list: async () => [], findById: async () => { throw new Error("unavailable"); },
  }, { prepare });
  await expect(command.handle({ input: { libraryId: "cloud", trackId: "song" } })).resolves.toMatchObject({ ok: false, error: { kind: "repository" } });
  expect(prepare).not.toHaveBeenCalled();
});
