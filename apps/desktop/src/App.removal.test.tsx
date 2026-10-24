import { afterEach, beforeEach, expect, it, vi } from "vitest";
import {
  act,
  cleanup,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "./App";
import { installTestRuntime, removeTestRuntime } from "./test-runtime";

const library = {
  id: "source",
  name: "Music",
  kind: "filesystem",
  location: "/music",
};
let invoke: ReturnType<typeof vi.fn>;
beforeEach(() => {
  HTMLDialogElement.prototype.showModal = function showModal() {
    this.setAttribute("open", "");
  };
  HTMLDialogElement.prototype.close = function close() {
    this.removeAttribute("open");
    this.dispatchEvent(new Event("close"));
  };
  vi.spyOn(HTMLMediaElement.prototype, "pause").mockImplementation(() => {});
  vi.spyOn(HTMLMediaElement.prototype, "load").mockImplementation(() => {});
  invoke = vi.fn().mockImplementation(async (command: string) => {
    if (command === "list_libraries") return [library];
    if (command === "dropbox_status")
      return { ok: true, value: { connected: false, accountId: null } };
    if (command === "remove_library") return { ok: true, value: { warning: null } };
    return [];
  });
  installTestRuntime({ invoke, openDirectory: vi.fn() });
});
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  removeTestRuntime();
});

it("confirms removing a source from Settings and updates Sources immediately", async () => {
  const user = userEvent.setup();
  render(<App />);
  await user.click(screen.getByRole("button", { name: "Settings" }));
  const sources = within(
    screen.getByRole("region", { name: "Indexed sources" }),
  );
  await user.click(
    await sources.findByRole("button", { name: "Remove Music" }),
  );
  const dialog = screen.getByRole("dialog", { name: "Remove Music?" });
  expect(dialog.textContent).toContain("music files stay where they are");
  expect(invoke).not.toHaveBeenCalledWith("remove_library", expect.anything());
  await user.click(within(dialog).getByRole("button", { name: "Cancel" }));
  expect(screen.queryByRole("dialog")).toBeNull();
  await user.click(sources.getByRole("button", { name: "Remove Music" }));
  await user.click(
    within(screen.getByRole("dialog")).getByRole("button", {
      name: "Remove source",
    }),
  );
  await waitFor(() =>
    expect(
      screen.queryByRole("button", { name: "Show tracks for Music" }),
    ).toBeNull(),
  );
  expect(invoke).toHaveBeenCalledWith("remove_library", {
    input: { libraryId: "source" },
  });
  expect(screen.getByRole("heading", { name: "Settings" })).toBeDefined();
});

it("offers removal beside a sidebar source and keeps it when removal fails", async () => {
  let attempts = 0;
  invoke.mockImplementation(async (command: string) => {
    if (command === "list_libraries") return [library];
    if (command === "remove_library") {
      attempts += 1;
      return attempts === 1
        ? {
            ok: false,
            error: { kind: "repository", message: "Disk is busy. Try again." },
          }
        : { ok: true, value: { warning: null } };
    }
    return [];
  });
  const user = userEvent.setup();
  render(<App />);
  await user.click(await screen.findByRole("button", { name: "Remove Music" }));
  await user.click(
    within(screen.getByRole("dialog")).getByRole("button", {
      name: "Remove source",
    }),
  );
  expect(screen.getByRole("alert").textContent).toContain("Disk is busy");
  expect(
    screen.getByRole("button", { name: "Show tracks for Music" }),
  ).toBeDefined();
  await user.click(
    within(screen.getByRole("dialog")).getByRole("button", {
      name: "Remove source",
    }),
  );
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(
    screen.queryByRole("button", { name: "Show tracks for Music" }),
  ).toBeNull();
});

it("does not restore source counts when an old refresh finishes after removal", async () => {
  let finishScan: ((value: { tracksScanned: number }) => void) | undefined;
  invoke.mockImplementation(async (command: string) => {
    if (command === "list_libraries") return [library];
    if (command === "dropbox_status")
      return { ok: true, value: { connected: false, accountId: null } };
    if (command === "scan_library")
      return new Promise((resolve) => {
        finishScan = resolve;
      });
    if (command === "list_tracks") return [{ id: "late-track" }];
    if (command === "remove_library") return { ok: true, value: { warning: null } };
    return [];
  });
  const user = userEvent.setup();
  render(<App />);
  await user.click(screen.getByRole("button", { name: "Settings" }));
  const sources = within(
    screen.getByRole("region", { name: "Indexed sources" }),
  );
  await user.click(sources.getByRole("button", { name: "Refresh Music" }));
  await user.click(sources.getByRole("button", { name: "Remove Music" }));
  await user.click(
    within(screen.getByRole("dialog")).getByRole("button", {
      name: "Remove source",
    }),
  );
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  finishScan?.({ tracksScanned: 1 });
  await waitFor(() =>
    expect(invoke).toHaveBeenCalledWith("list_tracks", { libraryId: "source" }),
  );
  expect(screen.getByText("0 libraries - 0 tracks")).toBeDefined();
  expect(screen.queryByText("Refreshed Music: 1 track.")).toBeNull();
});


it("shows a completed-removal cleanup warning after dismissing the confirmation", async () => {
  invoke.mockImplementation(async (command: string) => {
    if (command === "list_libraries") return [library];
    if (command === "remove_library") return { ok: true, value: { warning: "Source removed. Restart Muzo to release its filesystem watcher." } };
    return [];
  });
  render(<App />);
  await userEvent.click(await screen.findByRole("button", { name: "Remove Music" }));
  await userEvent.click(within(screen.getByRole("dialog")).getByRole("button", { name: "Remove source" }));
  expect(await screen.findByRole("status")).toHaveProperty("textContent", "Source removed. Restart Muzo to release its filesystem watcher.");
  expect(screen.queryByRole("dialog")).toBeNull();
});


it("discards a typed Dropbox refresh failure after its source was removed", async () => {
  let finishSync: ((value: unknown) => void) | undefined;
  invoke.mockImplementation(async (command: string) => {
    if (command === "list_libraries") return [{ ...library, kind: "dropbox" }];
    if (command === "dropbox_status") return { ok: true, value: { connected: false, accountId: null } };
    if (command === "dropbox_sync_library") return new Promise(resolve => { finishSync = resolve; });
    if (command === "remove_library") return { ok: true, value: { warning: null } };
    return [];
  });
  render(<App />);
  await userEvent.click(screen.getByRole("button", { name: "Settings" }));
  const sources = within(screen.getByRole("region", { name: "Indexed sources" }));
  await userEvent.click(sources.getByRole("button", { name: "Refresh Music" }));
  await userEvent.click(sources.getByRole("button", { name: "Remove Music" }));
  await userEvent.click(within(screen.getByRole("dialog")).getByRole("button", { name: "Remove source" }));
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  await act(async () => finishSync?.({ ok: false, error: { kind: "remote", message: "Stale Dropbox error" } }));
  expect(screen.queryByText("Stale Dropbox error")).toBeNull();
});

it("marks retained playlist entries unavailable immediately after sidebar removal", async () => {
  let removed = false;
  invoke.mockImplementation(async (command: string) => {
    if (command === "list_libraries") return [library];
    if (command === "list_playlists") return [{ id: "playlist", name: "Evening", entries: [{ id: "entry", trackId: "track", position: 0, trackTitle: removed ? null : "Song", trackLibraryId: removed ? null : "source", trackArtist: "Artist", trackDurationSeconds: 120 }] }];
    if (command === "remove_library") { removed = true; return { ok: true, value: { warning: null } }; }
    return [];
  });
  render(<App />);
  await userEvent.click(screen.getByRole("button", { name: "Playlists" }));
  expect(await screen.findByRole("button", { name: "Play Song from Evening" })).toBeDefined();
  await userEvent.click(screen.getByRole("button", { name: "Remove Music" }));
  await userEvent.click(within(screen.getByRole("dialog")).getByRole("button", { name: "Remove source" }));
  expect(await screen.findByText("Unavailable source")).toBeDefined();
  expect(screen.getByRole("button", { name: "Track track is unavailable in Evening" }).hasAttribute("disabled")).toBe(true);
  expect(screen.queryByRole("button", { name: "Play Song from Evening" })).toBeNull();
});
