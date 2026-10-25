// @vitest-environment node
import { expect, it, vi } from "vitest";
import { createApplicationUpdates } from "./application-updates";

it("announces a newer release and requires separate download and restart requests", async () => {
  const backend = {
    check: vi.fn(async () => ({
      ok: true as const,
      value: { version: "0.0.3", automatic: true },
    })),
    download: vi.fn(async () => ({ ok: true as const, value: undefined })),
    install: vi.fn(async () => ({
      ok: true as const,
      value: "installing" as const,
    })),
    openRelease: vi.fn(async () => ({ ok: true as const, value: undefined })),
    subscribe: vi.fn(() => () => {}),
  };
  const updates = createApplicationUpdates({
    installedVersion: "0.0.2",
    backend,
  });
  expect((await updates.check()).ok).toBe(true);
  expect(updates.getStatus()).toEqual({
    state: "available",
    version: "0.0.3",
    automatic: true,
  });
  expect(backend.download).not.toHaveBeenCalled();
  expect((await updates.install()).ok).toBe(false);
  expect(backend.install).not.toHaveBeenCalled();
  expect((await updates.download()).ok).toBe(true);
  expect(updates.getStatus()).toEqual({ state: "ready", version: "0.0.3" });
  expect(backend.install).not.toHaveBeenCalled();
  expect((await updates.install()).ok).toBe(true);
  expect(backend.download).toHaveBeenCalledWith("0.0.3");
  expect(backend.install).toHaveBeenCalledWith("0.0.3");
  expect(updates.getStatus()).toEqual({
    state: "installing",
    version: "0.0.3",
  });
});

it.each(["0.0.1", "0.0.2"])(
  "does not offer an equal or older release: %s",
  async (version) => {
    const backend = {
      check: async () => ({
        ok: true as const,
        value: { version, automatic: true },
      }),
      download: vi.fn(),
      install: vi.fn(),
      openRelease: vi.fn(),
      subscribe: vi.fn(() => () => {}),
    };
    const updates = createApplicationUpdates({
      installedVersion: "0.0.2",
      backend,
    });
    expect((await updates.check()).ok).toBe(true);
    expect(updates.getStatus()).toEqual({ state: "current" });
  },
);

it.each(["0.0.3-beta.1", "v0.0.3", "invalid", "9999999999999999999999.0.0"])(
  "rejects non-stable or malformed release metadata: %s",
  async (version) => {
    const backend = {
      check: async () => ({
        ok: true as const,
        value: { version, automatic: true },
      }),
      download: vi.fn(),
      install: vi.fn(),
      openRelease: vi.fn(),
      subscribe: vi.fn(() => () => {}),
    };
    const updates = createApplicationUpdates({
      installedVersion: "0.0.2",
      backend,
    });
    expect((await updates.check()).ok).toBe(false);
    expect(updates.getStatus().state).toBe("failed");
    expect(backend.download).not.toHaveBeenCalled();
  },
);

it("coalesces checks and keeps a download from being replaced by a background check", async () => {
  let finish:
    | ((value: {
        ok: true;
        value: { version: string; automatic: boolean };
      }) => void)
    | undefined;
  let finishDownload:
    ((value: { ok: true; value: undefined }) => void) | undefined;
  const backend = {
    check: vi.fn(
      () =>
        new Promise<{
          ok: true;
          value: { version: string; automatic: boolean };
        }>((resolve) => {
          finish = resolve;
        }),
    ),
    download: vi.fn(
      () =>
        new Promise<{ ok: true; value: undefined }>((resolve) => {
          finishDownload = resolve;
        }),
    ),
    install: vi.fn(),
    openRelease: vi.fn(),
    subscribe: vi.fn(() => () => {}),
  };
  const updates = createApplicationUpdates({
    installedVersion: "0.0.2",
    backend,
  });
  const first = updates.check();
  const second = updates.check();
  expect(backend.check).toHaveBeenCalledTimes(1);
  finish?.({ ok: true, value: { version: "0.0.3", automatic: true } });
  await Promise.all([first, second]);
  const download = updates.download();
  expect((await updates.check()).ok).toBe(false);
  expect((await updates.download()).ok).toBe(false);
  expect(updates.getStatus().state).toBe("downloading");
  finishDownload?.({ ok: true, value: undefined });
  await download;
  expect((await updates.check()).ok).toBe(false);
  expect(updates.getStatus().state).toBe("ready");
});

it("reports bounded progress and installation failures and releases its subscription", async () => {
  let report:
    | ((
        event: import("./interfaces/update-backend").UpdateBackendEvent,
      ) => void)
    | undefined;
  let finish: ((value: { ok: true; value: undefined }) => void) | undefined;
  const unsubscribe = vi.fn();
  const backend = {
    check: async () => ({
      ok: true as const,
      value: { version: "0.0.3", automatic: true },
    }),
    download: () =>
      new Promise<{ ok: true; value: undefined }>((resolve) => {
        finish = resolve;
      }),
    install: async () => ({ ok: true as const, value: "installing" as const }),
    openRelease: vi.fn(),
    subscribe: (
      listener: (
        event: import("./interfaces/update-backend").UpdateBackendEvent,
      ) => void,
    ) => {
      report = listener;
      return unsubscribe;
    },
  };
  const updates = createApplicationUpdates({
    installedVersion: "0.0.2",
    backend,
  });
  await updates.check();
  const download = updates.download();
  report?.({ kind: "progress", percent: 48 });
  expect(updates.getStatus()).toEqual({
    state: "downloading",
    version: "0.0.3",
    progress: 48,
  });
  report?.({ kind: "progress", percent: Number.NaN });
  report?.({ kind: "progress", percent: 150 });
  expect(updates.getStatus()).toMatchObject({ progress: 100 });
  finish?.({ ok: true, value: undefined });
  await download;
  report?.({ kind: "progress", percent: 5 });
  expect(updates.getStatus().state).toBe("ready");
  await updates.install();
  report?.({ kind: "installFailed" });
  expect(updates.getStatus().state).toBe("failed");
  updates.dispose();
  expect(unsubscribe).toHaveBeenCalledOnce();
});

it("keeps an unsigned macOS release as an explicit manual download", async () => {
  const backend = {
    check: async () => ({
      ok: true as const,
      value: { version: "0.0.3", automatic: false },
    }),
    download: vi.fn(),
    install: vi.fn(),
    openRelease: vi.fn(async () => ({ ok: true as const, value: undefined })),
    subscribe: vi.fn(() => () => {}),
  };
  const updates = createApplicationUpdates({
    installedVersion: "0.0.2",
    backend,
  });
  await updates.check();
  await updates.download();
  expect(backend.openRelease).toHaveBeenCalledWith("0.0.3");
  expect(backend.download).not.toHaveBeenCalled();
  expect((await updates.install()).ok).toBe(false);
});
