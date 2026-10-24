import { afterEach, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { installTestRuntime, removeTestRuntime } from "./test-runtime";
import DropboxSettings from "./dropbox-settings";

afterEach(() => { cleanup(); removeTestRuntime(); });

it("connects and disconnects Dropbox from Settings without displaying credentials", async () => {
  const invoke = vi.fn().mockImplementation(async (command: string) => {
    if (command === "dropbox_status" || command === "dropbox_disconnect") return { ok: true, value: { connected: false, accountId: null } };
    if (command === "dropbox_begin") return { ok: true, value: { pending: true } };
    return { ok: true, value: { connected: true, accountId: "account-1" } };
  });
  installTestRuntime({ invoke, openDirectory: async () => null });
  const user = userEvent.setup();
  render(<DropboxSettings />);
  await user.type(await screen.findByLabelText("Dropbox app key"), "app-key");
  await user.click(screen.getByRole("button", { name: "Connect Dropbox" }));
  await user.type(await screen.findByLabelText("Authorization code"), "auth-code");
  await user.click(screen.getByRole("button", { name: "Finish connecting" }));
  expect(await screen.findByText("Connected to Dropbox.")).toBeDefined();
  expect(invoke).toHaveBeenCalledWith("dropbox_begin", { input: { clientId: "app-key" } });
  expect(invoke).toHaveBeenCalledWith("dropbox_complete", { input: { code: "auth-code" } });
  await user.click(screen.getByRole("button", { name: "Disconnect Dropbox" }));
  expect(await screen.findByRole("button", { name: "Connect Dropbox" })).toBeDefined();
});
