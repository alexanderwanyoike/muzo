import { afterEach, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import DropboxFolderPicker from "./dropbox-folder-picker";
import { installTestRuntime, removeTestRuntime } from "./test-runtime";

afterEach(() => {
  cleanup();
  removeTestRuntime();
});

it("selects folders across locations and adds all of them with their folder names", async () => {
  const invoke = vi.fn(
    async (
      command: string,
      args?: { input?: { path?: string; location?: string; name?: string } },
    ) => {
      if (command === "dropbox_list_folders")
        return {
          ok: true,
          value: {
            addedPaths: [],
            folders:
              args?.input?.path === "/Music"
                ? [{ id: "jazz", name: "Jazz", path: "/Music/Jazz" }]
                : [
                    { id: "music", name: "Music", path: "/Music" },
                    { id: "podcasts", name: "Podcasts", path: "/Podcasts" },
                  ],
          },
        };
      return {
        ok: true,
        value: {
          id: args?.input?.location,
          name: args?.input?.name,
          location: args?.input?.location,
          kind: "dropbox",
        },
      };
    },
  );
  installTestRuntime({ invoke, openDirectory: async () => null });
  const onAdded = vi.fn();
  const user = userEvent.setup();
  render(
    <DropboxFolderPicker
      accountId="account"
      onAdded={onAdded}
      onClose={() => {}}
    />,
  );
  await user.click(await screen.findByRole("button", { name: "Open Music" }));
  await user.click(
    await screen.findByRole("checkbox", { name: "Select Jazz" }),
  );
  await user.click(screen.getByRole("button", { name: "Open Dropbox root" }));
  await user.click(
    await screen.findByRole("checkbox", { name: "Select Podcasts" }),
  );
  expect(screen.getByText("2 folders selected")).toBeDefined();
  await user.click(screen.getByRole("button", { name: "Add 2 folders" }));
  await waitFor(() => expect(onAdded).toHaveBeenCalledTimes(2));
  expect(invoke).toHaveBeenCalledWith("dropbox_add_library", {
    input: { name: "Jazz", location: "/Music/Jazz", accountId: "account" },
  });
  expect(invoke).toHaveBeenCalledWith("dropbox_add_library", {
    input: { name: "Podcasts", location: "/Podcasts", accountId: "account" },
  });
  expect(
    screen.getByRole("region", { name: "Dropbox folder browser" }),
  ).toBeDefined();
  expect(screen.getByText("Added 2 folders.")).toBeDefined();
});
