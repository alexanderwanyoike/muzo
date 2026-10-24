// @vitest-environment node
import { expect, it, vi } from "vitest";
import { createApplicationUpdates } from "./application-updates";

it("announces a newer release and requires separate download and restart requests", async () => {
  const backend = {
    check: vi.fn(async () => ({ ok: true as const, value: { version: "0.0.3", automatic: true } })),
    download: vi.fn(async () => ({ ok: true as const, value: undefined })),
    install: vi.fn(async () => ({ ok: true as const, value: "installing" as const })),
    openRelease: vi.fn(async () => ({ ok: true as const, value: undefined })),
    subscribe: vi.fn(() => () => {}),
  };
  const updates = createApplicationUpdates({ installedVersion: "0.0.2", backend });
  expect((await updates.check()).ok).toBe(true);
  expect(updates.getStatus()).toEqual({ state: "available", version: "0.0.3", automatic: true });
  expect(backend.download).not.toHaveBeenCalled();
  expect((await updates.install()).ok).toBe(false);
  expect(backend.install).not.toHaveBeenCalled();
  expect((await updates.download()).ok).toBe(true);
  expect(updates.getStatus()).toEqual({ state: "ready", version: "0.0.3" });
  expect(backend.install).not.toHaveBeenCalled();
  expect((await updates.install()).ok).toBe(true);
  expect(backend.download).toHaveBeenCalledWith("0.0.3");
  expect(backend.install).toHaveBeenCalledWith("0.0.3");
  expect(updates.getStatus()).toEqual({ state: "installing", version: "0.0.3" });
});


it.each(["0.0.1", "0.0.2"])("does not offer an equal or older release: %s", async (version) => {
  const backend = { check: async () => ({ ok: true as const, value: { version, automatic: true } }), download: vi.fn(), install: vi.fn(), openRelease: vi.fn(), subscribe: vi.fn(() => () => {}) };
  const updates = createApplicationUpdates({ installedVersion: "0.0.2", backend });
  expect((await updates.check()).ok).toBe(true);
  expect(updates.getStatus()).toEqual({ state: "current" });
});

it.each(["0.0.3-beta.1", "v0.0.3", "invalid", "9999999999999999999999.0.0"])("rejects non-stable or malformed release metadata: %s", async (version) => {
  const backend = { check: async () => ({ ok: true as const, value: { version, automatic: true } }), download: vi.fn(), install: vi.fn(), openRelease: vi.fn(), subscribe: vi.fn(() => () => {}) };
  const updates = createApplicationUpdates({ installedVersion: "0.0.2", backend });
  expect((await updates.check()).ok).toBe(false);
  expect(updates.getStatus().state).toBe("failed");
  expect(backend.download).not.toHaveBeenCalled();
});
