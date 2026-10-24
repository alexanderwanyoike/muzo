import { afterEach, beforeEach, expect, it, vi } from "vitest";
import {
  cleanup,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "./App";
import { installTestRuntime, removeTestRuntime } from "./test-runtime";

beforeEach(() => {
  localStorage.clear();
  vi.spyOn(HTMLMediaElement.prototype, "pause").mockImplementation(() => {});
  vi.spyOn(HTMLMediaElement.prototype, "load").mockImplementation(() => {});
  installTestRuntime({
    invoke: vi.fn().mockResolvedValue([]),
    openDirectory: vi.fn(),
  });
});
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  removeTestRuntime();
  localStorage.clear();
});

it("remembers a material, dark appearance and accent across a restart", async () => {
  const user = userEvent.setup();
  const first = render(<App />);
  await user.click(screen.getByRole("button", { name: "Settings" }));
  await user.click(screen.getByRole("radio", { name: "Brushed aluminium" }));
  await user.click(screen.getByRole("radio", { name: "Dark" }));
  await user.click(screen.getByRole("radio", { name: "Sage" }));
  expect(document.documentElement.dataset).toMatchObject({
    theme: "aluminium",
    mode: "dark",
    accent: "sage",
  });
  first.unmount();
  render(<App />);
  await user.click(screen.getByRole("button", { name: "Settings" }));
  await waitFor(() =>
    expect(
      screen.getByRole<HTMLInputElement>("radio", {
        name: "Brushed aluminium",
      }).checked,
    ).toBe(true),
  );
  expect(
    screen.getByRole<HTMLInputElement>("radio", { name: "Dark" }).checked,
  ).toBe(true);
  expect(
    screen.getByRole<HTMLInputElement>("radio", { name: "Sage" }).checked,
  ).toBe(true);
});

it("keeps appearance usable when saving preferences is unavailable", async () => {
  const user = userEvent.setup();
  render(<App />);
  await user.click(screen.getByRole("button", { name: "Settings" }));
  vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
    throw new Error("full");
  });
  await user.click(screen.getByRole("radio", { name: "Warm paper" }));
  expect(document.documentElement.dataset.theme).toBe("paper");
  expect(
    within(screen.getByRole("region", { name: "Appearance" })).getByRole(
      "alert",
    ).textContent,
  ).toContain("could not be saved");
});

it("recovers from malformed saved preferences without applying untrusted values", async () => {
  localStorage.setItem(
    "muzo.appearance.v1",
    '{"theme":"unknown","mode":"dark","accent":"sage"}',
  );
  render(<App />);
  await userEvent.click(screen.getByRole("button", { name: "Settings" }));
  expect(document.documentElement.dataset).toMatchObject({
    theme: "studio",
    mode: "light",
    accent: "cobalt",
  });
  expect(
    within(screen.getByRole("region", { name: "Appearance" })).getByRole(
      "alert",
    ).textContent,
  ).toContain("could not be read");
});

it.each(["Carbon fibre", "Walnut", "Porcelain"])("remembers the %s material across a restart", async (material) => {
  const first = render(<App />);
  await userEvent.click(screen.getByRole("button", { name: "Settings" }));
  await userEvent.click(screen.getByRole("radio", { name: material }));
  await userEvent.click(screen.getByRole("radio", { name: "Dark" }));
  first.unmount();
  render(<App />);
  await userEvent.click(screen.getByRole("button", { name: "Settings" }));
  expect(screen.getByRole<HTMLInputElement>("radio", { name: material }).checked).toBe(true);
  expect(screen.getByRole<HTMLInputElement>("radio", { name: "Dark" }).checked).toBe(true);
});

it.each(["Teal", "Rose", "Slate", "Violet", "Ice"])("remembers the %s accent across a restart", async (accent) => {
  const first = render(<App />);
  await userEvent.click(screen.getByRole("button", { name: "Settings" }));
  await userEvent.click(screen.getByRole("radio", { name: accent }));
  first.unmount();
  render(<App />);
  await userEvent.click(screen.getByRole("button", { name: "Settings" }));
  expect(screen.getByRole<HTMLInputElement>("radio", { name: accent }).checked).toBe(true);
});
