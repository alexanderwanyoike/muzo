import { afterEach, expect, it, vi } from "vitest";
import { cleanup, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "./App";
import { installTestRuntime, removeTestRuntime } from "./test-runtime";

afterEach(() => {
  cleanup();
  removeTestRuntime();
  localStorage.clear();
});

it("uses the existing sidebar and remembered divider for Playlists", async () => {
  installTestRuntime({
    invoke: vi.fn().mockImplementation(async (command: string) => command === "list_playlists" ? [
      { id: "road", name: "Road Trip", entries: [] },
      { id: "night", name: "Quiet hours", entries: [] },
    ] : []),
    openDirectory: async () => null,
  });
  render(<App />);
  const sourcesDivider = await screen.findByRole("separator", { name: "Sources" });
  sourcesDivider.focus();
  await userEvent.keyboard("{ArrowRight}");
  await userEvent.click(screen.getByRole("button", { name: "Playlists", exact: true }));
  const panel = await screen.findByRole("region", { name: "Playlists" });
  expect(screen.getByRole("complementary").contains(panel)).toBe(true);
  expect(screen.queryByRole("region", { name: "Sources" })).toBeNull();
  expect(screen.getAllByRole("separator")).toHaveLength(1);
  const divider = screen.getByRole("separator", { name: "Playlists" });
  expect(divider.getAttribute("aria-valuenow")).toBe("248");
  await userEvent.click(await within(panel).findByRole("button", { name: "Open playlist Quiet hours" }));
  expect(screen.getByRole("heading", { name: "Quiet hours" })).toBeDefined();
  await userEvent.type(screen.getByRole("searchbox", { name: "Search playlists" }), "Quiet");
  await userEvent.click(screen.getByRole("button", { name: "Songs", exact: true }));
  expect(screen.getByRole("region", { name: "Sources" })).toBeDefined();
  expect(screen.getByRole("separator", { name: "Sources" }).getAttribute("aria-valuenow")).toBe("248");
  await userEvent.click(screen.getByRole("button", { name: "Playlists", exact: true }));
  expect(await screen.findByRole("heading", { name: "Quiet hours" })).toBeDefined();
  expect(screen.getByRole<HTMLInputElement>("searchbox", { name: "Search playlists" }).value).toBe("Quiet");
});
