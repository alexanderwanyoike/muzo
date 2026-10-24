// @vitest-environment node
import { expect, it, vi } from "vitest";

import { TrackArtworkService } from "./track-artwork-service";

const emptyCache = {
  find: async () => ({ ok: true as const, value: null }),
  save: async () => ({ ok: true as const, value: undefined }),
};
const logger = { error: vi.fn() };

it("continues serving artwork after an unexpected cache failure", async () => {
  const find = vi
    .fn()
    .mockRejectedValueOnce(new Error("Cache failed unexpectedly"))
    .mockResolvedValue({ ok: true, value: null });
  const service = new TrackArtworkService(
    {
      find: async () => ({
        libraryId: "local",
        trackId: "track",
        path: "/song",
        kind: "filesystem",
        revision: "1:2",
      }),
    },
    { read: async () => ({ data: new Uint8Array([1]), format: "image/png" }) },
    { create: () => "data:image/png;base64,cover" },
    { read: vi.fn() },
    { accessFor: vi.fn() },
    { find, save: emptyCache.save },
    logger,
  );
  expect(await service.get("local", "track")).toMatchObject({ ok: false });
  expect(await service.get("local", "track")).toEqual({
    ok: true,
    value: "data:image/png;base64,cover",
  });
});

it("shows a persisted Dropbox thumbnail offline without reading credentials or media again", async () => {
  const accessFor = vi.fn().mockResolvedValue({
    ok: false,
    error: { kind: "notConnected", message: "Offline" },
  });
  const read = vi.fn();
  const service = new TrackArtworkService(
    {
      find: async () => ({
        libraryId: "cloud",
        trackId: "song",
        path: "/Music/song.flac",
        kind: "dropbox",
        revision: "1:2",
      }),
    },
    { read },
    { create: vi.fn() },
    { read },
    { accessFor },
    {
      find: async () => ({
        ok: true,
        value: { uri: "data:image/png;base64,cached" },
      }),
      save: emptyCache.save,
    },
    logger,
  );
  expect(await service.get("cloud", "song")).toEqual({
    ok: true,
    value: "data:image/png;base64,cached",
  });
  expect(accessFor).not.toHaveBeenCalled();
  expect(read).not.toHaveBeenCalled();
});

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
  source.revision = "1:3";
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

it("keeps artwork available when the persistent cache cannot be read or written", async () => {
  const errors = { error: vi.fn() };
  const service = new TrackArtworkService(
    {
      find: async () => ({
        libraryId: "local",
        trackId: "track",
        path: "/song",
        kind: "filesystem",
        revision: "1:2",
      }),
    },
    { read: async () => ({ data: new Uint8Array([1]), format: "image/png" }) },
    { create: () => "data:image/png;base64,cover" },
    { read: vi.fn() },
    { accessFor: vi.fn() },
    {
      find: async () => ({
        ok: false,
        error: { kind: "storage", message: "Read unavailable" },
      }),
      save: async () => ({
        ok: false,
        error: { kind: "storage", message: "Write unavailable" },
      }),
    },
    errors,
  );
  expect(await service.get("local", "track")).toEqual({
    ok: true,
    value: "data:image/png;base64,cover",
  });
  expect(errors.error).toHaveBeenCalledTimes(2);
});

it("invalidates persisted artwork after a scanned file revision changes", async () => {
  let revision = "1:2";
  const values = new Map<string, { uri: string | null }>();
  const persisted = {
    find: async (key: string) => ({
      ok: true as const,
      value: values.get(key) ?? null,
    }),
    save: async (key: string, uri: string | null) => {
      values.set(key, { uri });
      return { ok: true as const, value: undefined };
    },
  };
  const read = vi
    .fn()
    .mockResolvedValue({ data: new Uint8Array([1]), format: "image/png" });
  const make = () =>
    new TrackArtworkService(
      {
        find: async () => ({
          libraryId: "local",
          trackId: "track",
          path: "/song",
          kind: "filesystem",
          revision,
        }),
      },
      { read },
      { create: () => `data:image/png;base64,cover${read.mock.calls.length}` },
      { read: vi.fn() },
      { accessFor: vi.fn() },
      persisted,
      logger,
    );
  expect(await make().get("local", "track")).toEqual({
    ok: true,
    value: "data:image/png;base64,cover1",
  });
  expect(await make().get("local", "track")).toEqual({
    ok: true,
    value: "data:image/png;base64,cover1",
  });
  expect(read).toHaveBeenCalledOnce();
  revision = "1:3";
  expect(await make().get("local", "track")).toEqual({
    ok: true,
    value: "data:image/png;base64,cover2",
  });
  expect(read).toHaveBeenCalledTimes(2);
});
