import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "./App";
import { installTestRuntime, removeTestRuntime } from "./test-runtime";

beforeEach(() => {
  localStorage.clear();
  vi.spyOn(HTMLMediaElement.prototype, "pause").mockImplementation(() => {});
  vi.spyOn(HTMLMediaElement.prototype, "load").mockImplementation(() => {});
  installTestRuntime({ invoke: vi.fn().mockResolvedValue([]), openDirectory: vi.fn() });
});
afterEach(() => { cleanup(); vi.restoreAllMocks(); removeTestRuntime(); localStorage.clear(); });

it("remembers a material, dark appearance and accent across a restart", async () => {
  const user = userEvent.setup();
  const first = render(<App />);
  await user.click(screen.getByRole("button", { name: "Settings" }));
  await user.click(screen.getByRole("radio", { name: "Brushed aluminium" }));
  await user.click(screen.getByRole("radio", { name: "Dark" }));
  await user.click(screen.getByRole("radio", { name: "Sage" }));
  expect(document.documentElement.dataset).toMatchObject({ theme: "aluminium", mode: "dark", accent: "sage" });
  first.unmount();
  render(<App />);
  await user.click(screen.getByRole("button", { name: "Settings" }));
  await waitFor(() => expect((screen.getByRole("radio", { name: "Brushed aluminium" }) as HTMLInputElement).checked).toBe(true));
  expect((screen.getByRole("radio", { name: "Dark" }) as HTMLInputElement).checked).toBe(true);
  expect((screen.getByRole("radio", { name: "Sage" }) as HTMLInputElement).checked).toBe(true);
});
