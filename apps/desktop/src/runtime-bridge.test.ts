import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { open as tauriOpen } from "@tauri-apps/plugin-dialog";
import { getRuntimeBridge, openDirectory } from "./runtime-bridge";

const mockedTauriInvoke = tauriInvoke as unknown as ReturnType<typeof vi.fn>;
const mockedTauriOpen = tauriOpen as unknown as ReturnType<typeof vi.fn>;

beforeEach(() => {
  mockedTauriInvoke.mockReset();
  mockedTauriOpen.mockReset();
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
    expect(mockedTauriInvoke).not.toHaveBeenCalled();
  });

  it("falls back to Tauri when no runtime is injected", async () => {
    mockedTauriInvoke.mockResolvedValue("pong");

    await expect(getRuntimeBridge().invoke("ping", { input: true })).resolves.toBe(
      "pong",
    );

    expect(mockedTauriInvoke).toHaveBeenCalledWith("ping", { input: true });
  });

  it("normalises the Tauri directory picker to a nullable string", async () => {
    mockedTauriOpen.mockResolvedValueOnce(["/tmp/one"]);

    await expect(openDirectory()).resolves.toBeNull();

    expect(mockedTauriOpen).toHaveBeenCalledWith({
      directory: true,
      multiple: false,
    });
  });
});
