import { afterEach, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "./App";
import { installTestRuntime, removeTestRuntime } from "./test-runtime";

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  removeTestRuntime();
});
it("adds and syncs multiple Dropbox folders while keeping Settings open", async () => {
  vi.spyOn(HTMLMediaElement.prototype, "pause").mockImplementation(() => {});
  vi.spyOn(HTMLMediaElement.prototype, "load").mockImplementation(() => {});
  const library = {
    id: "cloud",
    name: "Cloud Music",
    kind: "dropbox",
    location: "/Music",
  };
  const invoke = vi
    .fn()
    .mockImplementation(
      async (
        command: string,
        args?: { input?: { name?: string; location?: string } },
      ) => {
        switch (command) {
          case "dropbox_status":
            return {
              ok: true,
              value: { connected: true, accountId: "account-1" },
            };
          case "dropbox_sync_status":
            return {
              ok: true,
              value: {
                intervalMinutes: 5,
                syncing: false,
                lastError: null,
                lastSyncedAtUnixSeconds: null,
              },
            };
          case "dropbox_list_folders":
            return {
              ok: true,
              value: {
                folders: [
                  { id: "music", name: "Music", path: "/Music" },
                  { id: "podcasts", name: "Podcasts", path: "/Podcasts" },
                ],
                addedPaths: [],
              },
            };
          case "dropbox_add_library":
            return {
              ok: true,
              value: {
                ...library,
                id: args?.input?.location,
                name: args?.input?.name,
                location: args?.input?.location,
              },
            };
          case "dropbox_sync_library":
            return { ok: true, value: { tracksScanned: 0 } };
          default:
            return [];
        }
      },
    );
  installTestRuntime({ invoke, openDirectory: async () => null });
  const user = userEvent.setup();
  render(<App />);
  await user.click(screen.getByRole("button", { name: "Settings" }));
  await user.click(
    await screen.findByRole("button", { name: "Browse Dropbox folders" }),
  );
  await user.click(
    await screen.findByRole("checkbox", { name: "Select Music" }),
  );
  await user.click(screen.getByRole("checkbox", { name: "Select Podcasts" }));
  await user.click(screen.getByRole("button", { name: "Add 2 folders" }));
  await waitFor(() =>
    expect(invoke).toHaveBeenCalledWith("dropbox_sync_library", {
      input: { libraryId: "/Music" },
    }),
  );
  await waitFor(() =>
    expect(invoke).toHaveBeenCalledWith("dropbox_sync_library", {
      input: { libraryId: "/Podcasts" },
    }),
  );
  expect(screen.getByRole("heading", { name: "Settings" })).toBeDefined();
  expect(
    screen.getByRole("button", { name: "Show tracks for Music" }),
  ).toBeDefined();
  expect(
    screen.getByRole("button", { name: "Show tracks for Podcasts" }),
  ).toBeDefined();
  expect(
    screen.getByRole("region", { name: "Dropbox folder browser" }),
  ).toBeDefined();
});
