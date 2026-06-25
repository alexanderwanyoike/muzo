import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "./App";
import type { TrackDto } from "./api";

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

const library = {
  id: "lib-1",
  name: "My Music",
  kind: "filesystem",
  location: "/home/user/Music",
};

const track: TrackDto = {
  id: "trk-1",
  libraryId: "lib-1",
  title: "Hotel California",
  artist: "Eagles",
  album: "Hotel California",
  trackNumber: 1,
  discNumber: 1,
  genre: "Rock",
  year: 1976,
  metadataOverridden: false,
  durationSeconds: 391,
  filePath: "/home/user/Music/Hotel California.mp3",
};

describe("App", () => {
  it("renders the Muzo heading", async () => {
    mockedInvoke.mockResolvedValue([]);
    render(<App />);
    expect(screen.getByRole("heading", { name: "Muzo" })).toBeDefined();
    await waitFor(() => expect(screen.getByText("No music yet")).toBeDefined());
  });

  it("keeps library import controls out of the main listening view", async () => {
    mockedInvoke.mockResolvedValue([]);

    render(<App />);

    await waitFor(() => expect(screen.getByText("No music yet")).toBeDefined());
    expect(screen.queryByRole("button", { name: /browse/i })).toBeNull();
    expect(screen.queryByRole("button", { name: /add library/i })).toBeNull();
  });

  it("shows library import controls in settings", async () => {
    const user = userEvent.setup();
    mockedInvoke.mockResolvedValue([]);

    render(<App />);

    await waitFor(() => expect(screen.getByText("No music yet")).toBeDefined());
    await user.click(screen.getByRole("button", { name: "Settings" }));

    expect(screen.getByRole("heading", { name: "Settings" })).toBeDefined();
    expect(screen.getByRole("button", { name: /browse/i })).toBeDefined();
    expect(screen.getByRole("button", { name: /add library/i })).toBeDefined();
  });

  it("loads libraries on mount and renders them", async () => {
    mockedInvoke.mockResolvedValue([library]);

    render(<App />);

    await waitFor(() => expect(screen.getByText("My Music")).toBeDefined());
  });

  it("does not fetch tracks until a library is expanded", async () => {
    mockedInvoke.mockImplementation((command: string) => {
      if (command === "list_libraries") return Promise.resolve([library]);
      if (command === "list_tracks") return Promise.resolve([track]);
      return Promise.resolve([]);
    });

    render(<App />);

    await waitFor(() => expect(screen.getByText("My Music")).toBeDefined());
    expect(mockedInvoke).not.toHaveBeenCalledWith("list_tracks", {
      libraryId: "lib-1",
    });

    await userEvent.click(
      screen.getByRole("button", { name: "Show tracks for My Music" }),
    );

    await waitFor(() =>
      expect(mockedInvoke).toHaveBeenCalledWith("list_tracks", {
        libraryId: "lib-1",
      }),
    );
    expect(screen.getByText("Hotel California")).toBeDefined();
  });

  it("refreshes the visible track list after scanning an expanded library", async () => {
    const user = userEvent.setup();
    mockedInvoke.mockImplementation((command: string) => {
      if (command === "list_libraries") return Promise.resolve([library]);
      if (command === "list_tracks") return Promise.resolve([]);
      if (command === "scan_library") return Promise.resolve({ tracksScanned: 1 });
      return Promise.resolve([]);
    });

    render(<App />);

    await waitFor(() => expect(screen.getByText("My Music")).toBeDefined());
    await user.click(
      screen.getByRole("button", { name: "Show tracks for My Music" }),
    );
    await waitFor(() => expect(screen.getByText(/no tracks yet/i)).toBeDefined());

    mockedInvoke.mockImplementation((command: string) => {
      if (command === "list_libraries") return Promise.resolve([library]);
      if (command === "list_tracks") return Promise.resolve([track]);
      if (command === "scan_library") return Promise.resolve({ tracksScanned: 1 });
      return Promise.resolve([]);
    });

    await user.click(
      screen.getByRole("button", { name: "Scan selected library" }),
    );

    await waitFor(() =>
      expect(screen.getByText("Hotel California")).toBeDefined(),
    );
  });
});
