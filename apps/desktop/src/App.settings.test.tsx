import { afterEach, expect, it, vi } from "vitest";
import { cleanup, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "./App";
import { installTestRuntime, removeTestRuntime } from "./test-runtime";

afterEach(() => {
  cleanup();
  removeTestRuntime();
  vi.restoreAllMocks();
  localStorage.clear();
});

it("uses compact source actions while retaining their full accessible names", async () => {
  vi.spyOn(HTMLMediaElement.prototype, "pause").mockImplementation(() => {});
  vi.spyOn(HTMLMediaElement.prototype, "load").mockImplementation(() => {});
  installTestRuntime({
    invoke: vi.fn().mockImplementation(async (command: string) => {
      if (command === "list_libraries")
        return [
          {
            id: "music",
            name: "Evening listening",
            kind: "filesystem",
            location: "/Music/Evening",
          },
        ];
      if (command === "dropbox_status")
        return { ok: true, value: { connected: false, accountId: null } };
      return [];
    }),
    openDirectory: async () => null,
  });
  render(<App />);
  await userEvent.click(screen.getByRole("button", { name: "Settings" }));
  const sources = within(
    await screen.findByRole("region", { name: "Indexed sources" }),
  );
  expect(
    (await sources.findByRole("button", { name: "Refresh Evening listening" }))
      .textContent,
  ).toBe("Refresh");
  expect(
    sources.getByRole("button", { name: "Remove Evening listening" })
      .textContent,
  ).toBe("Remove");
});
