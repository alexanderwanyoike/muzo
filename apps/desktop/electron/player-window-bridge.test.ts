// @vitest-environment node
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { runInNewContext } from "node:vm";
import { expect, it, vi } from "vitest";

it("exposes a narrow Result-based player window command through the isolated preload", async () => {
  const exposeInMainWorld = vi.fn();
  const invoke = vi.fn(async () => ({ ok: true, value: undefined }));
  runInNewContext(
    readFileSync(
      fileURLToPath(new URL("./preload.cjs", import.meta.url)),
      "utf8",
    ),
    {
      require: () => ({
        contextBridge: { exposeInMainWorld },
        ipcRenderer: { invoke },
      }),
    },
  );
  const bridge = exposeInMainWorld.mock.calls[0][1] as {
    setPlayerWindowMode: (mode: string) => Promise<unknown>;
  };
  expect(typeof bridge.setPlayerWindowMode).toBe("function");
  expect(await bridge.setPlayerWindowMode("compact")).toEqual({
    ok: true,
    value: undefined,
  });
  expect(invoke).toHaveBeenCalledWith("muzo:player-window-mode", "compact");
});
