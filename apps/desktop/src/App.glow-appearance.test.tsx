import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, render, screen, within } from "@testing-library/react";
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
const choices = [
  "Glow green",
  "Glow cyan",
  "Glow blue",
  "Glow violet",
  "Glow pink",
  "Glow orange",
].flatMap((accent) => ["Light", "Dark"].map((mode) => ({ accent, mode })));
it.each(choices)(
  "restores $accent in $mode mode and can return to a classic accent",
  async ({ accent, mode }) => {
    const user = userEvent.setup();
    const first = render(<App />);
    await user.click(screen.getByRole("button", { name: "Settings" }));
    await user.click(screen.getByRole("radio", { name: mode }));
    await user.click(
      within(screen.getByRole("group", { name: "Glow accents" })).getByRole(
        "radio",
        { name: accent },
      ),
    );
    const selected = document.documentElement.dataset.accent;
    first.unmount();
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Settings" }));
    expect(
      screen.getByRole<HTMLInputElement>("radio", { name: accent }).checked,
    ).toBe(true);
    expect(document.documentElement.dataset).toMatchObject({
      theme: "studio",
      mode: mode.toLowerCase(),
      accent: selected,
    });
    await user.click(screen.getByRole("radio", { name: "Cobalt" }));
    expect(document.documentElement.dataset.accent).toBe("cobalt");
    expect(
      screen.getByRole<HTMLInputElement>("radio", { name: accent }).checked,
    ).toBe(false);
  },
);
