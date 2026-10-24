import { afterEach, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { installTestRuntime, removeTestRuntime } from "./test-runtime";
import DropboxLibrarySettings from "./dropbox-library-settings";

afterEach(() => {
  cleanup();
  removeTestRuntime();
});
it("adds a Dropbox folder and changes the background sync interval", async () => {
  const library = {
    id: "cloud",
    name: "Cloud",
    kind: "dropbox",
    location: "/Music",
  };
  const invoke = vi.fn().mockImplementation(async (command: string) => {
    if (command === "dropbox_sync_status")
      return {
        ok: true,
        value: {
          intervalMinutes: 5,
          syncing: false,
          lastError: null,
          lastSyncedAtUnixSeconds: null,
        },
      };
    if (command === "dropbox_set_sync_interval")
      return { ok: true, value: { intervalMinutes: 15 } };
    return { ok: true, value: library };
  });
  installTestRuntime({ invoke, openDirectory: async () => null });
  const onAdded = vi.fn();
  const user = userEvent.setup();
  render(<DropboxLibrarySettings onAdded={onAdded} />);
  await user.type(screen.getByLabelText("Dropbox library name"), "Cloud");
  await user.type(screen.getByLabelText("Dropbox folder"), "/Music");
  await user.click(screen.getByRole("button", { name: "Add Dropbox library" }));
  await waitFor(() => expect(onAdded).toHaveBeenCalledWith(library));
  expect(invoke).toHaveBeenCalledWith("dropbox_add_library", {
    input: { name: "Cloud", location: "/Music" },
  });
  await user.selectOptions(
    await screen.findByLabelText("Sync Dropbox every"),
    "15",
  );
  expect(invoke).toHaveBeenCalledWith("dropbox_set_sync_interval", {
    input: { minutes: 15 },
  });
});
