// @vitest-environment node
import { expect, it, vi } from "vitest";
const emptyCache = {
  find: async () => ({ ok: true as const, value: null }),
  save: async () => ({ ok: true as const, value: undefined }),
};
const logger = { error: vi.fn() };

import { TrackArtworkService } from "./track-artwork-service";

it("reads Dropbox artwork with account-bound credentials without returning them to the renderer", async () => {
  const source = {
    libraryId: "cloud",
    trackId: "track",
    path: "/Music/song.flac",
    kind: "dropbox" as const,
    revision: "1:2",
  };
  const read = vi
    .fn()
    .mockResolvedValue({ format: "image/png", data: new Uint8Array([1]) });
  const accessFor = vi.fn().mockResolvedValue({
    ok: true,
    value: {
      accessToken: "private-token",
      library: {
        id: "cloud",
        kind: "dropbox",
        location: "/Music",
        name: "Music",
      },
    },
  });
  const service = new TrackArtworkService(
    { find: async () => source },
    { read: vi.fn() },
    { create: () => "data:image/png;base64,cover" },
    { read },
    { accessFor },
    emptyCache,
    logger,
  );
  expect(await service.get("cloud", "track")).toEqual({
    ok: true,
    value: "data:image/png;base64,cover",
  });
  expect(accessFor).toHaveBeenCalledWith("cloud");
  expect(read).toHaveBeenCalledWith("private-token", "/Music/song.flac");
  accessFor.mockResolvedValueOnce({
    ok: false,
    error: { kind: "notConnected", message: "Connect the owning account." },
  });
  expect(await service.get("cloud", "track")).toMatchObject({
    ok: false,
    error: { kind: "notConnected" },
  });
  expect(read).toHaveBeenCalledOnce();
});

it("shares in-flight cover reads and reloads them when the stored file revision changes", async () => {
  let revision = "1:2";
  let finish:
    | ((value: { format: string; data: Uint8Array }) => void)
    | undefined;
  const picture = { format: "image/png", data: new Uint8Array([1]) };
  const read = vi
    .fn()
    .mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          finish = resolve;
        }),
    )
    .mockResolvedValue(picture);
  const service = new TrackArtworkService(
    {
      find: async () => ({
        libraryId: "local",
        trackId: "track",
        path: "/music/song.flac",
        kind: "filesystem",
        revision,
      }),
    },
    { read },
    { create: () => "data:image/png;base64,cover" },
    { read: vi.fn() },
    { accessFor: vi.fn() },
    emptyCache,
    logger,
  );
  const first = service.get("local", "track");
  const second = service.get("local", "track");
  await vi.waitFor(() => expect(read).toHaveBeenCalled());
  finish?.(picture);
  await Promise.all([first, second]);
  expect(read).toHaveBeenCalledOnce();
  await service.get("local", "track");
  expect(read).toHaveBeenCalledOnce();
  revision = "1:3";
  await service.get("local", "track");
  expect(read).toHaveBeenCalledTimes(2);
});
