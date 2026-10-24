import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import PlaylistPanel from "./playlist-panel";
import { installTestRuntime, removeTestRuntime } from "./test-runtime";

let mockedInvoke: ReturnType<typeof vi.fn>;
let mockedArtwork: ReturnType<typeof vi.fn>;

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
        trackLibraryId: "lib-1",
        trackDurationSeconds: 391,
        position: 0,
      },
      {
        id: "entry-2",
        trackId: "track-2",
        trackTitle: null,
        trackArtist: null,
        trackLibraryId: null,
        trackDurationSeconds: null,
        position: 1,
      },
    ],
  },
];

beforeEach(() => {
  mockedInvoke = vi.fn();
  mockedArtwork = vi.fn().mockResolvedValue({ ok: true, value: null });
  installTestRuntime({
    invoke: vi.fn((command: string, args?: unknown) => (command === "get_track_artwork" ? mockedArtwork : mockedInvoke)(command, args)),
    openDirectory: vi.fn().mockResolvedValue(null),
  });
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
    mockedInvoke.mockResolvedValueOnce([]).mockResolvedValueOnce({
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
    expect(screen.getByLabelText<HTMLInputElement>("Playlist name").value).toBe(
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
    expect(screen.getByLabelText<HTMLInputElement>("Playlist name").value).toBe(
      "Morning",
    );
  });

  it("removes a playlist entry and updates the visible count", async () => {
    const user = userEvent.setup();
    mockedInvoke.mockResolvedValueOnce(playlists).mockResolvedValueOnce({
      id: "playlist-1",
      name: "Road Trip",
      entries: [playlists[0].entries[1]],
    });

    render(<PlaylistPanel />);

    await waitFor(() =>
      expect(screen.getByText("Hotel California")).toBeDefined(),
    );
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

    await waitFor(() =>
      expect(screen.getByText("Hotel California")).toBeDefined(),
    );
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
    mockedInvoke.mockResolvedValueOnce(playlists).mockResolvedValueOnce({
      id: "playlist-1",
      name: "Road Trip",
      entries: [playlists[0].entries[1], playlists[0].entries[0]],
    });

    render(<PlaylistPanel />);

    await waitFor(() =>
      expect(screen.getByText("Hotel California")).toBeDefined(),
    );
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

    await waitFor(() =>
      expect(screen.getByText("Hotel California")).toBeDefined(),
    );

    expect(
      screen.getByRole<HTMLButtonElement>("button", {
        name: "Move Hotel California up in Road Trip",
      }).disabled,
    ).toBe(true);
    expect(
      screen.getByRole<HTMLButtonElement>("button", {
        name: "Move track-2 down in Road Trip",
      }).disabled,
    ).toBe(true);
  });

  it("keeps playlist entry order and shows reorder failures", async () => {
    const user = userEvent.setup();
    mockedInvoke
      .mockResolvedValueOnce(playlists)
      .mockRejectedValueOnce(new Error("Could not reorder playlist entries"));

    render(<PlaylistPanel />);

    await waitFor(() =>
      expect(screen.getByText("Hotel California")).toBeDefined(),
    );
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

  it("plays a resolved playlist entry", async () => {
    const user = userEvent.setup();
    const onPlayTrack = vi.fn();
    mockedInvoke.mockResolvedValueOnce(playlists);

    render(
      <PlaylistPanel
        currentTrackId={null}
        isPlaying={false}
        onPlayTrack={onPlayTrack}
        onToggleCurrentTrack={() => {}}
      />,
    );

    await waitFor(() =>
      expect(screen.getByText("Hotel California")).toBeDefined(),
    );
    await user.click(
      screen.getByRole("button", {
        name: "Play Hotel California from Road Trip",
      }),
    );

    expect(onPlayTrack).toHaveBeenCalledWith({
      id: "track-1",
      libraryId: "lib-1",
      title: "Hotel California",
      artist: "Eagles",
      album: null,
      trackNumber: null,
      discNumber: null,
      genre: null,
      year: null,
      metadataOverridden: false,
      durationSeconds: 391,
      filePath: "",
    });
  });

  it("toggles the current playlist entry", async () => {
    const user = userEvent.setup();
    const onPlayTrack = vi.fn();
    const onToggleCurrentTrack = vi.fn();
    mockedInvoke.mockResolvedValueOnce(playlists);

    render(
      <PlaylistPanel
        currentTrackId="track-1"
        isPlaying
        onPlayTrack={onPlayTrack}
        onToggleCurrentTrack={onToggleCurrentTrack}
      />,
    );

    await waitFor(() =>
      expect(screen.getByText("Hotel California")).toBeDefined(),
    );
    await user.click(
      screen.getByRole("button", {
        name: "Pause Hotel California from Road Trip",
      }),
    );

    expect(onToggleCurrentTrack).toHaveBeenCalledOnce();
    expect(onPlayTrack).not.toHaveBeenCalled();
  });

  it("disables playback for playlist entries whose track no longer exists", async () => {
    mockedInvoke.mockResolvedValueOnce(playlists);

    render(
      <PlaylistPanel
        currentTrackId={null}
        isPlaying={false}
        onPlayTrack={() => {}}
        onToggleCurrentTrack={() => {}}
      />,
    );

    await waitFor(() => expect(screen.getByText("track-2")).toBeDefined());

    expect(
      screen.getByRole<HTMLButtonElement>("button", {
        name: "Track track-2 is unavailable in Road Trip",
      }).disabled,
    ).toBe(true);
  });
  it("queues a playlist in order while skipping unavailable entries", async () => {
    const user = userEvent.setup();
    mockedInvoke.mockResolvedValueOnce(playlists);
    const onPlayQueue = vi.fn();
    render(<PlaylistPanel onPlayQueue={onPlayQueue} />);
    await user.click(
      await screen.findByRole("button", { name: "Play Road Trip" }),
    );
    expect(onPlayQueue).toHaveBeenCalledWith(
      [
        {
          entryId: "entry-1",
          track: expect.objectContaining({ id: "track-1", libraryId: "lib-1" }),
        },
      ],
      0,
    );
  });

  it("starts at a selected duplicate entry rather than toggling another occurrence", async () => {
    const user = userEvent.setup();
    const repeated = {
      ...playlists[0],
      entries: [
        playlists[0].entries[0],
        { ...playlists[0].entries[0], id: "repeat", position: 1 },
      ],
    };
    mockedInvoke.mockResolvedValueOnce([repeated]);
    const onPlayQueue = vi.fn();
    const onToggleCurrentTrack = vi.fn();
    render(
      <PlaylistPanel
        currentTrackId="track-1"
        currentQueueEntryId="entry-1"
        isPlaying
        onPlayQueue={onPlayQueue}
        onToggleCurrentTrack={onToggleCurrentTrack}
      />,
    );
    await user.click(
      await screen.findByRole("button", {
        name: "Play Hotel California from Road Trip",
      }),
    );
    expect(onToggleCurrentTrack).not.toHaveBeenCalled();
    expect(onPlayQueue).toHaveBeenCalledWith(
      [
        {
          entryId: "entry-1",
          track: expect.objectContaining({ id: "track-1" }),
        },
        {
          entryId: "repeat",
          track: expect.objectContaining({ id: "track-1" }),
        },
      ],
      1,
    );
  });
});

function entryLabels(): string[] {
  return screen
    .getAllByTestId("playlist-entry-label")
    .map((element) => element.textContent ?? "");
}

it("shows covers for available playlist entries and skips unavailable sources", async () => {
  vi.stubGlobal("IntersectionObserver", vi.fn((callback: IntersectionObserverCallback) => ({ observe: () => callback([{ isIntersecting: true }] as IntersectionObserverEntry[], { disconnect: vi.fn() } as unknown as IntersectionObserver), disconnect: vi.fn() })));
  mockedInvoke.mockResolvedValue(playlists);
  mockedArtwork.mockResolvedValue({ ok: true, value: "data:image/png;base64,cover" });
  render(<PlaylistPanel />);
  expect(await screen.findByRole("img", { name: "Artwork for Hotel California" })).toBeDefined();
  expect(mockedArtwork).toHaveBeenCalledWith("get_track_artwork", { input: { libraryId: "lib-1", trackId: "track-1" } });
  expect(mockedArtwork).not.toHaveBeenCalledWith("get_track_artwork", { input: { libraryId: null, trackId: "track-2" } });
  vi.unstubAllGlobals();
});
