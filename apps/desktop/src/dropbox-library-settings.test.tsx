import { afterEach, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { installTestRuntime, removeTestRuntime } from "./test-runtime";
import DropboxLibrarySettings from "./dropbox-library-settings";

afterEach(() => {
  cleanup();
  removeTestRuntime();
});
it("chooses folders from Dropbox without entering a path or library name", async () => {
  const invoke = vi.fn().mockImplementation(async (command: string) => {
    if (command === "dropbox_list_folders")
      return {
        ok: true,
        value: {
          folders: [{ id: "music", name: "Music", path: "/Music" }],
          addedPaths: [],
        },
      };
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
    return {
      ok: true,
      value: {
        id: "cloud",
        name: "Music",
        kind: "dropbox",
        location: "/Music",
      },
    };
  });
  installTestRuntime({ invoke, openDirectory: async () => null });
  const onAdded = vi.fn();
  const user = userEvent.setup();
  render(<DropboxLibrarySettings accountId="account-1" onAdded={onAdded} />);
  await user.click(
    screen.getByRole("button", { name: "Browse Dropbox folders" }),
  );
  await user.click(
    await screen.findByRole("checkbox", { name: "Select Music" }),
  );
  await user.click(screen.getByRole("button", { name: "Add 1 folder" }));
  await waitFor(() =>
    expect(onAdded).toHaveBeenCalledWith({
      id: "cloud",
      name: "Music",
      kind: "dropbox",
      location: "/Music",
    }),
  );
  expect(invoke).toHaveBeenCalledWith("dropbox_list_folders", {
    input: { accountId: "account-1", path: "/" },
  });
  expect(invoke).toHaveBeenCalledWith("dropbox_add_library", {
    input: { name: "Music", location: "/Music", accountId: "account-1" },
  });
  expect(screen.queryByLabelText("Dropbox folder")).toBeNull();
  expect(screen.queryByLabelText("Dropbox library name")).toBeNull();
});

it("returns keyboard focus to Browse when closing the folder browser", async () => {
  const invoke = vi.fn().mockResolvedValue({
    ok: true,
    value: { folders: [], addedPaths: [], intervalMinutes: 5 },
  });
  installTestRuntime({ invoke, openDirectory: async () => null });
  const user = userEvent.setup();
  render(<DropboxLibrarySettings accountId="account" onAdded={() => {}} />);
  await user.click(
    screen.getByRole("button", { name: "Browse Dropbox folders" }),
  );
  await user.click(
    await screen.findByRole("button", { name: "Close folder browser" }),
  );
  expect(
    screen.queryByRole("region", { name: "Dropbox folder browser" }),
  ).toBeNull();
  expect(document.activeElement).toBe(
    screen.getByRole("button", { name: "Browse Dropbox folders" }),
  );
});
