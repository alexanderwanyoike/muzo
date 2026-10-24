import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "./App";
import { installTestRuntime, removeTestRuntime } from "./test-runtime";

const library = { id: "source", name: "Music", kind: "filesystem", location: "/music" };
let invoke: ReturnType<typeof vi.fn>;
beforeEach(() => {
  vi.spyOn(HTMLMediaElement.prototype, "pause").mockImplementation(() => {});
  vi.spyOn(HTMLMediaElement.prototype, "load").mockImplementation(() => {});
  invoke = vi.fn().mockImplementation(async (command: string) => {
    if (command === "list_libraries") return [library];
    if (command === "dropbox_status") return { ok: true, value: { connected: false, accountId: null } };
    if (command === "remove_library") return { ok: true, value: undefined };
    return [];
  });
  installTestRuntime({ invoke, openDirectory: vi.fn() });
});
afterEach(() => { cleanup(); vi.restoreAllMocks(); removeTestRuntime(); });

it("confirms removing a source from Settings and updates Sources immediately", async () => {
  const user = userEvent.setup();
  render(<App />);
  await user.click(screen.getByRole("button", { name: "Settings" }));
  await user.click(await screen.findByRole("button", { name: "Remove Music" }));
  const dialog = screen.getByRole("dialog", { name: "Remove Music?" });
  expect(dialog.textContent).toContain("music files stay where they are");
  expect(invoke).not.toHaveBeenCalledWith("remove_library", expect.anything());
  await user.click(within(dialog).getByRole("button", { name: "Cancel" }));
  expect(screen.queryByRole("dialog")).toBeNull();
  await user.click(screen.getByRole("button", { name: "Remove Music" }));
  await user.click(within(screen.getByRole("dialog")).getByRole("button", { name: "Remove source" }));
  await waitFor(() => expect(screen.queryByRole("button", { name: "Show tracks for Music" })).toBeNull());
  expect(invoke).toHaveBeenCalledWith("remove_library", { input: { libraryId: "source" } });
  expect(screen.getByRole("heading", { name: "Settings" })).toBeDefined();
});
