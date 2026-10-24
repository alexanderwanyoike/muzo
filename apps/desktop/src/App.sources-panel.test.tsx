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

it("resizes the sidebar with the keyboard and remembers its width on reopening", async () => {
  installTestRuntime({ invoke: vi.fn().mockResolvedValue([]), openDirectory: async () => null });
  const view = render(<App />);
  const divider = await screen.findByRole("separator", { name: "Sources" });
  expect(divider.getAttribute("aria-valuenow")).toBe("240");
  divider.focus();
  await userEvent.keyboard("{ArrowRight}");
  expect(divider.getAttribute("aria-valuenow")).toBe("248");
  view.unmount();
  render(<App />);
  expect((await screen.findByRole("separator", { name: "Sources" })).getAttribute("aria-valuenow")).toBe("248");
});
