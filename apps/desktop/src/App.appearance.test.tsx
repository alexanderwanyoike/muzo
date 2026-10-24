import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor, within } from "@testing-library/react";
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

it("keeps appearance usable when saving preferences is unavailable", async () => {
  const user = userEvent.setup();
  render(<App />);
  await user.click(screen.getByRole("button", { name: "Settings" }));
  vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => { throw new Error("full"); });
  await user.click(screen.getByRole("radio", { name: "Warm paper" }));
  expect(document.documentElement.dataset.theme).toBe("paper");
  expect(within(screen.getByRole("region", { name: "Appearance" })).getByRole("alert").textContent).toContain("could not be saved");
});

it("recovers from malformed saved preferences without applying untrusted values", async () => {
  localStorage.setItem("muzo.appearance.v1", '{"theme":"unknown","mode":"dark","accent":"sage"}');
  render(<App />);
  await userEvent.click(screen.getByRole("button", { name: "Settings" }));
  expect(document.documentElement.dataset).toMatchObject({ theme: "studio", mode: "light", accent: "cobalt" });
  expect(within(screen.getByRole("region", { name: "Appearance" })).getByRole("alert").textContent).toContain("could not be read");
});
