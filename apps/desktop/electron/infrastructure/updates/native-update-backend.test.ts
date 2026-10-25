// @vitest-environment node
import { EventEmitter } from "node:events";
import { expect, it, vi } from "vitest";
import { success } from "../../application/result";
import { createNativeUpdateBackend } from "./native-update-backend";

function fixture() {
  const events = new EventEmitter();
  const client = { autoDownload: true, autoInstallOnAppQuit: true, allowPrerelease: true, allowDowngrade: true, disableDifferentialDownload: false,
    on: events.on.bind(events),
    checkForUpdates: vi.fn(async () => ({ updateInfo: { version: "0.0.3" } })),
    downloadUpdate: vi.fn(async () => ["/verified/Muzo.AppImage"]), quitAndInstall: vi.fn(),
  };
  const options = { enabled: true, automatic: true, createClient: vi.fn(() => client), checkRelease: vi.fn(async () => success({ version: "0.0.3" })), confirmRestart: vi.fn(async () => true), installDownloaded: vi.fn(async () => success(undefined)), openRelease: vi.fn(async () => {}) };
  return { client, options, events, backend: createNativeUpdateBackend(options) };
}
it("verifies the selected release before download and installs only its verified payload after confirmation", async () => {
  const { backend, client, options } = fixture();
  expect(await backend.check()).toEqual({ ok: true, value: { version: "0.0.3", automatic: true } });
  expect(options.createClient).not.toHaveBeenCalled();
  expect((await backend.install("0.0.3")).ok).toBe(false);
  expect((await backend.download("0.0.3")).ok).toBe(true);
  expect(client.autoDownload).toBe(false); expect(client.autoInstallOnAppQuit).toBe(false);
  expect(client.allowPrerelease).toBe(false); expect(client.allowDowngrade).toBe(false);
  expect(client.disableDifferentialDownload).toBe(true);
  expect((await backend.install("0.0.3")).ok).toBe(true);
  expect(options.confirmRestart).toHaveBeenCalledWith("0.0.3");
  expect(options.installDownloaded).toHaveBeenCalledWith("/verified/Muzo.AppImage", client);
});
it("does not download a release that changed after the notice was shown", async () => {
  const { backend, client } = fixture();
  client.checkForUpdates.mockResolvedValueOnce({ updateInfo: { version: "0.0.4" } });
  expect((await backend.download("0.0.3")).ok).toBe(false);
  expect(client.downloadUpdate).not.toHaveBeenCalled();
});
