// @vitest-environment node
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { runInNewContext } from "node:vm";
import { expect, it, vi } from "vitest";
import { installDesktopGlass } from "./install-desktop-glass";
import type { Result } from "./application/result";

it("allows only the owning player main frame to change desktop glass through the isolated preload", async () => {
  const handle = vi.fn();
  const frame = {};
  let event = { senderFrame: frame, sender: { mainFrame: frame } };
  let closed = false;
  const window = {
    isDestroyed: () => closed,
    setBackgroundColor: vi.fn(),
    setVibrancy: vi.fn(),
    setBackgroundMaterial: vi.fn(),
  };
  const findWindow = vi.fn().mockReturnValue(window);
  installDesktopGlass({ handle }, findWindow, () => ({
    platform: "linux",
    release: "6.8.0",
    reducedTransparency: false,
  }));
  expect(handle).toHaveBeenCalledWith(
    "muzo:desktop-glass",
    expect.any(Function),
  );
  const command = handle.mock.calls[0][1] as (
    event: unknown,
    enabled: unknown,
  ) => Result<boolean>;
  const exposeInMainWorld = vi.fn();
  runInNewContext(
    readFileSync(
      fileURLToPath(new URL("./preload.cjs", import.meta.url)),
      "utf8",
    ),
    {
      require: () => ({
        contextBridge: { exposeInMainWorld },
        ipcRenderer: {
          invoke: async (channel: string, enabled: unknown) => {
            expect(channel).toBe("muzo:desktop-glass");
            return command(event, enabled);
          },
        },
      }),
    },
  );
  const bridge = exposeInMainWorld.mock.calls[0][1] as {
    setDesktopGlass: (enabled: unknown) => Promise<Result<boolean>>;
  };
  expect(await bridge.setDesktopGlass(true)).toEqual({ ok: true, value: true });
  expect(window.setBackgroundColor).toHaveBeenLastCalledWith("#00000000");
  expect(await bridge.setDesktopGlass(false)).toEqual({
    ok: true,
    value: false,
  });
  expect(window.setBackgroundColor).toHaveBeenLastCalledWith("#ffffff");
  expect(await bridge.setDesktopGlass("transparent")).toMatchObject({
    ok: false,
    error: { kind: "invalidInput" },
  });
  findWindow.mockClear();
  event = { ...event, senderFrame: {} };
  expect(await bridge.setDesktopGlass(true)).toMatchObject({
    ok: false,
    error: { kind: "authorization" },
  });
  expect(findWindow).not.toHaveBeenCalled();
  event = { ...event, senderFrame: frame };
  closed = true;
  expect(await bridge.setDesktopGlass(true)).toMatchObject({ ok: false });
  findWindow.mockReturnValue(null);
  expect(await bridge.setDesktopGlass(true)).toMatchObject({ ok: false });
  expect(window.setBackgroundColor).toHaveBeenCalledTimes(2);
});
