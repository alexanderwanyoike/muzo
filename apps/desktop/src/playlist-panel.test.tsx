import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import PlaylistPanel from "./playlist-panel";
import { installTestRuntime, removeTestRuntime } from "./test-runtime";

let mockedInvoke: ReturnType<typeof vi.fn>;

const playlists = [
  {
    id: "playlist-1",
    name: "Road Trip",
    entries: [
      {
        id: "entry-1",
        trackId: "track-1",
        trackTitle: "Hotel California",
        trackArtist: "Eagles",
        position: 0,
      },
      {
        id: "entry-2",
        trackId: "track-2",
        trackTitle: null,
        trackArtist: null,
        position: 1,
      },
    ],
  },
];

beforeEach(() => {
  const runtime = installTestRuntime({
    invoke: vi.fn(),
    openDirectory: vi.fn().mockResolvedValue(null),
  });
  mockedInvoke = runtime.invoke as ReturnType<typeof vi.fn>;
});

afterEach(() => {
  removeTestRuntime();
});

describe("PlaylistPanel", () => {
  it("lists persisted playlists with track counts", async () => {
    mockedInvoke.mockResolvedValueOnce(playlists);

    render(<PlaylistPanel />);

    await waitFor(() =>
      expect(mockedInvoke).toHaveBeenCalledWith("list_playlists", undefined),
    );
    expect(screen.getByText("Road Trip")).toBeDefined();
    expect(screen.getByText("2 tracks")).toBeDefined();
    expect(screen.getByText("Hotel California")).toBeDefined();
    expect(screen.getByText("Eagles")).toBeDefined();
    expect(screen.getByText("track-2")).toBeDefined();
  });

  it("shows an empty state for playlists without entries", async () => {
    mockedInvoke.mockResolvedValueOnce([
      {
        id: "playlist-2",
        name: "Late Night",
        entries: [],
      },
    ]);

    render(<PlaylistPanel />);

    await waitFor(() =>
      expect(screen.getByText("No tracks in this playlist.")).toBeDefined(),
    );
  });

  it("shows an empty state when there are no playlists", async () => {
    mockedInvoke.mockResolvedValueOnce([]);

    render(<PlaylistPanel />);

    await waitFor(() =>
      expect(screen.getByText("No playlists yet")).toBeDefined(),
    );
  });

  it("creates a playlist with a trimmed name and appends it to the list", async () => {
    const user = userEvent.setup();
    mockedInvoke
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce({
        id: "playlist-2",
        name: "Late Night",
        entries: [],
      });

    render(<PlaylistPanel />);

    await waitFor(() =>
      expect(screen.getByText("No playlists yet")).toBeDefined(),
    );
    await user.type(screen.getByLabelText("Playlist name"), "  Late Night  ");
    await user.click(screen.getByRole("button", { name: "Create playlist" }));

    await waitFor(() =>
      expect(mockedInvoke).toHaveBeenCalledWith("create_playlist", {
        input: { name: "Late Night" },
      }),
    );
    expect(screen.getByText("Late Night")).toBeDefined();
    expect(screen.getByText("0 tracks")).toBeDefined();
    expect((screen.getByLabelText("Playlist name") as HTMLInputElement).value).toBe(
      "",
    );
  });

  it("rejects blank names before calling the backend", async () => {
    const user = userEvent.setup();
    mockedInvoke.mockResolvedValueOnce([]);

    render(<PlaylistPanel />);

    await waitFor(() =>
      expect(screen.getByText("No playlists yet")).toBeDefined(),
    );
    await user.type(screen.getByLabelText("Playlist name"), "   ");
    await user.click(screen.getByRole("button", { name: "Create playlist" }));

    expect(screen.getByRole("alert").textContent).toContain(
      "Enter a playlist name",
    );
    expect(mockedInvoke).toHaveBeenCalledTimes(1);
  });

  it("surfaces load failures", async () => {
    mockedInvoke.mockRejectedValueOnce(new Error("Could not read playlists"));

    render(<PlaylistPanel />);

    await waitFor(() =>
      expect(screen.getByRole("alert").textContent).toContain(
        "Could not read playlists",
      ),
    );
  });

  it("keeps the draft and shows create failures", async () => {
    const user = userEvent.setup();
    mockedInvoke
      .mockResolvedValueOnce([])
      .mockRejectedValueOnce(new Error("Could not create playlist"));

    render(<PlaylistPanel />);

    await waitFor(() =>
      expect(screen.getByText("No playlists yet")).toBeDefined(),
    );
    await user.type(screen.getByLabelText("Playlist name"), "Morning");
    await user.click(screen.getByRole("button", { name: "Create playlist" }));

    await waitFor(() =>
      expect(screen.getByRole("alert").textContent).toContain(
        "Could not create playlist",
      ),
    );
    expect((screen.getByLabelText("Playlist name") as HTMLInputElement).value).toBe(
      "Morning",
    );
  });

  it("removes a playlist entry and updates the visible count", async () => {
    const user = userEvent.setup();
    mockedInvoke
      .mockResolvedValueOnce(playlists)
      .mockResolvedValueOnce({
        id: "playlist-1",
        name: "Road Trip",
        entries: [playlists[0].entries[1]],
      });

    render(<PlaylistPanel />);

    await waitFor(() => expect(screen.getByText("Hotel California")).toBeDefined());
    await user.click(
      screen.getByRole("button", {
        name: "Remove Hotel California from Road Trip",
      }),
    );

    await waitFor(() =>
      expect(mockedInvoke).toHaveBeenCalledWith("remove_playlist_entry", {
        input: {
          playlistId: "playlist-1",
          entryId: "entry-1",
        },
      }),
    );
    expect(screen.queryByText("Hotel California")).toBeNull();
    expect(screen.getByText("1 track")).toBeDefined();
  });

  it("keeps the entry visible and shows remove failures", async () => {
    const user = userEvent.setup();
    mockedInvoke
      .mockResolvedValueOnce(playlists)
      .mockRejectedValueOnce(new Error("Could not remove playlist entry"));

    render(<PlaylistPanel />);

    await waitFor(() => expect(screen.getByText("Hotel California")).toBeDefined());
    await user.click(
      screen.getByRole("button", {
        name: "Remove Hotel California from Road Trip",
      }),
    );

    await waitFor(() =>
      expect(screen.getByRole("alert").textContent).toContain(
        "Could not remove playlist entry",
      ),
    );
    expect(screen.getByText("Hotel California")).toBeDefined();
  });

  it("moves a playlist entry up and updates the visible order", async () => {
    const user = userEvent.setup();
    mockedInvoke
      .mockResolvedValueOnce(playlists)
      .mockResolvedValueOnce({
        id: "playlist-1",
        name: "Road Trip",
        entries: [playlists[0].entries[1], playlists[0].entries[0]],
      });

    render(<PlaylistPanel />);

    await waitFor(() => expect(screen.getByText("Hotel California")).toBeDefined());
    await user.click(
      screen.getByRole("button", {
        name: "Move track-2 up in Road Trip",
      }),
    );

    await waitFor(() =>
      expect(mockedInvoke).toHaveBeenCalledWith("reorder_playlist_entries", {
        input: {
          playlistId: "playlist-1",
          orderedEntryIds: ["entry-2", "entry-1"],
        },
      }),
    );
    expect(entryLabels()).toEqual(["track-2", "Hotel California"]);
  });

  it("disables playlist entry moves at the list boundaries", async () => {
    mockedInvoke.mockResolvedValueOnce(playlists);

    render(<PlaylistPanel />);

    await waitFor(() => expect(screen.getByText("Hotel California")).toBeDefined());

    expect(
      (
        screen.getByRole("button", {
          name: "Move Hotel California up in Road Trip",
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(true);
    expect(
      (
        screen.getByRole("button", {
          name: "Move track-2 down in Road Trip",
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(true);
  });

  it("keeps playlist entry order and shows reorder failures", async () => {
    const user = userEvent.setup();
    mockedInvoke
      .mockResolvedValueOnce(playlists)
      .mockRejectedValueOnce(new Error("Could not reorder playlist entries"));

    render(<PlaylistPanel />);

    await waitFor(() => expect(screen.getByText("Hotel California")).toBeDefined());
    await user.click(
      screen.getByRole("button", {
        name: "Move track-2 up in Road Trip",
      }),
    );

    await waitFor(() =>
      expect(screen.getByRole("alert").textContent).toContain(
        "Could not reorder playlist entries",
      ),
    );
    expect(entryLabels()).toEqual(["Hotel California", "track-2"]);
  });
});

function entryLabels(): string[] {
  return screen
    .getAllByTestId("playlist-entry-label")
    .map((element) => element.textContent ?? "");
}
