// @vitest-environment node
import { afterEach, expect, it, vi } from "vitest";
import { DropboxLibraryPoller } from "./dropbox-library-poller";

afterEach(() => vi.useRealTimers());
it("syncs on startup and at the persisted interval, recovers from failure, and stops cleanly", async () => {
  vi.useFakeTimers();
  let interval = 5;
  const syncAll = vi.fn().mockResolvedValueOnce({ ok: false, error: { kind: "remote", message: "Offline" } })
    .mockResolvedValue({ ok: true, value: { librariesSynced: 1, failures: [] } });
  const poller = new DropboxLibraryPoller({ syncAll }, { intervalMinutes: async () => interval, setIntervalMinutes: async (minutes) => { interval = minutes; } }, () => 1000);
  poller.start();
  await vi.advanceTimersByTimeAsync(0);
  expect(syncAll).toHaveBeenCalledTimes(1);
  expect(await poller.status()).toMatchObject({ ok: true, value: { lastError: "Offline", intervalMinutes: 5 } });
  expect(await poller.setIntervalMinutes(1)).toEqual({ ok: true, value: { intervalMinutes: 1 } });
  await vi.advanceTimersByTimeAsync(60000);
  expect(syncAll).toHaveBeenCalledTimes(2);
  expect(await poller.status()).toMatchObject({ ok: true, value: { lastError: null, lastSyncedAtUnixSeconds: 1000 } });
  expect(await poller.setIntervalMinutes(0)).toMatchObject({ ok: false, error: { kind: "invalidInput" } });
  poller.stop();
  await vi.advanceTimersByTimeAsync(60000);
  expect(syncAll).toHaveBeenCalledTimes(2);
});
