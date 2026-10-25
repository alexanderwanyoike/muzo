// @vitest-environment node
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { runInNewContext } from "node:vm";
import { expect, it, vi } from "vitest";
import type { RuntimeBridge } from "../src/runtime-bridge";

it("returns a typed updater failure when the desktop transport is unavailable", async () => {
 let bridge: RuntimeBridge | undefined;
 const ipcRenderer = {invoke: vi.fn(async () => {throw new Error("No IPC handler");}), on: vi.fn(), removeListener: vi.fn()};
 runInNewContext(await readFile(fileURLToPath(new URL("./preload.cjs", import.meta.url)), "utf8"), {require: () => ({contextBridge: {exposeInMainWorld: (_key: string, exposed: RuntimeBridge) => {bridge = exposed;}}, ipcRenderer})});
 expect(await bridge?.updates?.getStatus()).toMatchObject({ok: false, error: {kind: "remote"}});
 expect(await bridge?.updates?.command("download")).toMatchObject({ok: false, error: {kind: "remote"}});
 const listener = vi.fn(); const unsubscribe = bridge?.updates?.subscribe(listener); expect(ipcRenderer.on).toHaveBeenCalledWith("muzo:update-status", expect.any(Function)); unsubscribe?.(); expect(ipcRenderer.removeListener).toHaveBeenCalledWith("muzo:update-status", ipcRenderer.on.mock.calls[0][1]);
});
