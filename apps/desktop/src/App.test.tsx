import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import App from "./App";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
  convertFileSrc: vi.fn((path: string) => `asset://${path}`),
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";

const mockedInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

beforeEach(() => {
  mockedInvoke.mockReset();
});

describe("App", () => {
  it("renders the Muzo heading", async () => {
    mockedInvoke.mockResolvedValue([]);
    render(<App />);
    expect(screen.getByRole("heading", { name: "Muzo" })).toBeDefined();
  });

  it("loads libraries on mount and renders them", async () => {
    mockedInvoke.mockResolvedValue([
      {
        id: "lib-1",
        name: "My Music",
        kind: "filesystem",
        location: "/home/user/Music",
      },
    ]);

    render(<App />);

    await waitFor(() => expect(screen.getByText("My Music")).toBeDefined());
  });
});
