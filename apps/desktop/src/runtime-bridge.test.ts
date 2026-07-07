import { beforeEach, describe, expect, it, vi } from "vitest";

import { getRuntimeBridge, invokeCommand, openDirectory } from "./runtime-bridge";

beforeEach(() => {
  delete window.__MUZO_RUNTIME__;
});

describe("runtime bridge", () => {
  it("uses an injected Electron runtime when one is available", async () => {
    const electronInvoke = vi.fn().mockResolvedValue("pong");
    window.__MUZO_RUNTIME__ = {
      invoke: electronInvoke,
      openDirectory: vi.fn(),
    };

    await expect(getRuntimeBridge().invoke("ping")).resolves.toBe("pong");

    expect(electronInvoke).toHaveBeenCalledWith("ping");
  });

  it("fails clearly when the desktop runtime is missing", async () => {
    await expect(invokeCommand("ping", { input: true })).rejects.toThrow(
      "Muzo desktop runtime is not available",
    );
  });

  it("delegates directory picking to the injected runtime", async () => {
    const openRuntimeDirectory = vi.fn().mockResolvedValue("/tmp/one");
    window.__MUZO_RUNTIME__ = {
      invoke: vi.fn(),
      openDirectory: openRuntimeDirectory,
    };

    await expect(openDirectory()).resolves.toBe("/tmp/one");

    expect(openRuntimeDirectory).toHaveBeenCalledOnce();
  });
});
