import { afterEach, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "./App";
import { installTestRuntime, removeTestRuntime } from "./test-runtime";

afterEach(() => { cleanup(); removeTestRuntime(); localStorage.clear(); vi.restoreAllMocks(); });

it("opens and focuses source setup from the panel add action", async () => {
  installTestRuntime({ invoke: vi.fn().mockImplementation(async (command: string) => command === "dropbox_status" ? { ok: true, value: { connected: false, accountId: null } } : []), openDirectory: async () => null });
  render(<App />);
  await screen.findByRole("region", { name: "Sources" });
  await userEvent.click(screen.getByRole("button", { name: "Add source" }));
  const setup = await screen.findByRole("region", { name: "Add sources" });
  await waitFor(() => expect(document.activeElement).toBe(setup));
  expect(screen.getByLabelText("Dropbox app key")).toBeDefined();
  expect(screen.getByRole("button", { name: "Browse" })).toBeDefined();
});
