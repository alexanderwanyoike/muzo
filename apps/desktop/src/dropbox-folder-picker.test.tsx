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
  const invoke = vi
    .fn()
    .mockImplementation(
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

it("keeps failed selections while adding the other folders and skips already-added folders", async () => {
  let failMusic = true;
  const invoke = vi
    .fn()
    .mockImplementation(
      async (
        command: string,
        args?: { input?: { name?: string; location?: string } },
      ) => {
        if (command === "dropbox_list_folders")
          return {
            ok: true,
            value: {
              addedPaths: ["/Archive"],
              folders: [
                { id: "music", name: "Music", path: "/Music" },
                { id: "podcasts", name: "Podcasts", path: "/Podcasts" },
                { id: "archive", name: "Archive", path: "/Archive" },
              ],
            },
          };
        if (args?.input?.location === "/Music" && failMusic)
          return {
            ok: false,
            error: {
              kind: "storage",
              message: "Could not add Music. Try again.",
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
  await user.click(
    await screen.findByRole("checkbox", { name: "Select Music" }),
  );
  await user.click(screen.getByRole("checkbox", { name: "Select Podcasts" }));
  await user.click(screen.getByRole("button", { name: "Add 2 folders" }));
  await waitFor(() => expect(onAdded).toHaveBeenCalledTimes(1));
  expect(
    screen.getByRole<HTMLInputElement>("checkbox", { name: "Select Music" })
      .checked,
  ).toBe(true);
  expect(
    screen.getByRole<HTMLInputElement>("checkbox", { name: "Select Podcasts" })
      .disabled,
  ).toBe(true);
  expect(
    screen.getByRole<HTMLInputElement>("checkbox", { name: "Select Archive" })
      .disabled,
  ).toBe(true);
  expect(screen.getByRole("alert").textContent).toContain(
    "Could not add Music",
  );
  failMusic = false;
  await user.click(screen.getByRole("button", { name: "Add 1 folder" }));
  await waitFor(() => expect(onAdded).toHaveBeenCalledTimes(2));
  expect(
    invoke.mock.calls.filter(
      ([command, args]) =>
        command === "dropbox_add_library" &&
        args.input.location === "/Podcasts",
    ),
  ).toHaveLength(1);
});

it("retries an unavailable location and can select the current folder when it has no subfolders", async () => {
  let musicAttempts = 0;
  const invoke = vi
    .fn()
    .mockImplementation(
      async (command: string, args?: { input?: { path?: string } }) => {
        if (command === "dropbox_add_library")
          return {
            ok: true,
            value: {
              id: "music",
              name: "Music",
              kind: "dropbox",
              location: "/Music",
            },
          };
        if (args?.input?.path === "/Music") {
          musicAttempts += 1;
          return musicAttempts === 1
            ? {
                ok: false,
                error: { kind: "remote", message: "Temporarily unavailable." },
              }
            : { ok: true, value: { folders: [], addedPaths: [] } };
        }
        return {
          ok: true,
          value: {
            folders: [{ id: "music", name: "Music", path: "/Music" }],
            addedPaths: [],
          },
        };
      },
    );
  installTestRuntime({ invoke, openDirectory: async () => null });
  const user = userEvent.setup();
  const onAdded = vi.fn();
  render(
    <DropboxFolderPicker
      accountId="account"
      onAdded={onAdded}
      onClose={() => {}}
    />,
  );
  await user.click(await screen.findByRole("button", { name: "Open Music" }));
  expect(await screen.findByRole("alert")).toHaveProperty(
    "textContent",
    "Temporarily unavailable.",
  );
  await user.click(screen.getByRole("button", { name: "Retry folders" }));
  expect(await screen.findByText("No subfolders here.")).toBeDefined();
  await user.click(
    screen.getByRole("checkbox", { name: "Select current folder" }),
  );
  await user.click(screen.getByRole("button", { name: "Add 1 folder" }));
  await waitFor(() => expect(onAdded).toHaveBeenCalledTimes(1));
  expect(invoke).toHaveBeenCalledWith("dropbox_add_library", {
    input: { name: "Music", location: "/Music", accountId: "account" },
  });
});
