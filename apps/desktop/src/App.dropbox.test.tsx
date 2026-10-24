import { afterEach, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "./App";
import { installTestRuntime, removeTestRuntime } from "./test-runtime";

afterEach(() => { cleanup(); vi.restoreAllMocks(); removeTestRuntime(); });
it("imports and immediately syncs a Dropbox folder from Settings", async () => {
  vi.spyOn(HTMLMediaElement.prototype, "pause").mockImplementation(() => {});
  vi.spyOn(HTMLMediaElement.prototype, "load").mockImplementation(() => {});
  const library = { id: "cloud", name: "Cloud Music", kind: "dropbox", location: "/Music" };
  const invoke = vi.fn().mockImplementation(async (command: string) => {
    switch (command) {
      case "dropbox_status": return { ok: true, value: { connected: true, accountId: "account-1" } };
      case "dropbox_sync_status": return { ok: true, value: { intervalMinutes: 5, syncing: false, lastError: null, lastSyncedAtUnixSeconds: null } };
      case "dropbox_add_library": return { ok: true, value: library };
      case "dropbox_sync_library": return { ok: true, value: { tracksScanned: 0 } };
      default: return [];
    }
  });
  installTestRuntime({ invoke, openDirectory: async () => null });
  const user = userEvent.setup();
  render(<App />);
  await user.click(screen.getByRole("button", { name: "Settings" }));
  await user.type(await screen.findByLabelText("Dropbox library name"), "Cloud Music");
  await user.type(screen.getByLabelText("Dropbox folder"), "/Music");
  await user.click(screen.getByRole("button", { name: "Add Dropbox library" }));
  await waitFor(() => expect(invoke).toHaveBeenCalledWith("dropbox_sync_library", { input: { libraryId: "cloud" } }));
  expect(await screen.findByRole("heading", { name: "Cloud Music" })).toBeDefined();
});
