import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import TrackList from "./TrackList";
import type { TrackDto } from "./api";
import { installTestRuntime, removeTestRuntime } from "./test-runtime";

let mockedInvoke: ReturnType<typeof vi.fn>;

const sampleTracks: TrackDto[] = [
  {
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
    filePath: "/music/Eagles/Hotel California.mp3",
  },
  {
    id: "trk-2",
    libraryId: "lib-1",
    title: "Take It Easy",
    artist: "Eagles",
    album: null,
    trackNumber: null,
    discNumber: null,
    genre: null,
    year: null,
    metadataOverridden: true,
    durationSeconds: 233,
    filePath: "/music/Eagles/Take It Easy.mp3",
  },
];

const samplePlaylists = [
  {
    id: "playlist-1",
    name: "Road Trip",
    entries: [],
  },
  {
    id: "playlist-2",
    name: "Late Night",
    entries: [],
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

describe("TrackList", () => {
  it("loads tracks for the given library on mount", async () => {
    mockedInvoke.mockResolvedValueOnce(sampleTracks);
    render(
      <TrackList
        libraryId="lib-1"
        currentTrackId={null}
        isPlaying={false}
        onPlayTrack={() => {}}
        onToggleCurrentTrack={() => {}}
      />,
    );

    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledWith("list_tracks", { libraryId: "lib-1" }));
    await waitFor(() => expect(screen.getByText("Hotel California")).toBeDefined());
    expect(screen.getByText("Take It Easy")).toBeDefined();
  });

  it("formats durations as m:ss", async () => {
    mockedInvoke.mockResolvedValueOnce(sampleTracks);
    render(
      <TrackList
        libraryId="lib-1"
        currentTrackId={null}
        isPlaying={false}
        onPlayTrack={() => {}}
        onToggleCurrentTrack={() => {}}
      />,
    );

    await waitFor(() => expect(screen.getByText("6:31")).toBeDefined());
    expect(screen.getByText("3:53")).toBeDefined();
  });

  it("shows the play count for each track", async () => {
    mockedInvoke.mockResolvedValueOnce(sampleTracks);
    render(
      <TrackList
        libraryId="lib-1"
        currentTrackId={null}
        isPlaying={false}
        onPlayTrack={() => {}}
        onToggleCurrentTrack={() => {}}
        playCounts={{ "trk-1": 3, "trk-2": 1 }}
      />,
    );

    await waitFor(() => expect(screen.getByText("3 plays")).toBeDefined());
    expect(screen.getByText("1 play")).toBeDefined();
  });

  it("shows album metadata and override state in the track row", async () => {
    mockedInvoke.mockResolvedValueOnce(sampleTracks);
    render(
      <TrackList
        libraryId="lib-1"
        currentTrackId={null}
        isPlaying={false}
        onPlayTrack={() => {}}
        onToggleCurrentTrack={() => {}}
      />,
    );

    await waitFor(() => expect(screen.getByText("Hotel California")).toBeDefined());

    expect(screen.getByText("Eagles - Hotel California")).toBeDefined();
    expect(screen.getByText("Eagles - Edited in Muzo")).toBeDefined();
  });

  it("saves edited metadata and reloads the track list", async () => {
    const user = userEvent.setup();
    mockedInvoke
      .mockResolvedValueOnce(sampleTracks)
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce([
        {
          ...sampleTracks[0],
          title: "Hotel California Live",
          metadataOverridden: true,
        },
      ]);
    render(
      <TrackList
        libraryId="lib-1"
        currentTrackId={null}
        isPlaying={false}
        onPlayTrack={() => {}}
        onToggleCurrentTrack={() => {}}
      />,
    );

    await waitFor(() => expect(screen.getByText("Hotel California")).toBeDefined());
    await user.click(screen.getByRole("button", { name: /edit hotel california/i }));
    await user.clear(screen.getByLabelText("Title"));
    await user.type(screen.getByLabelText("Title"), "Hotel California Live");
    await user.click(screen.getByRole("button", { name: "Save metadata" }));

    await waitFor(() =>
      expect(mockedInvoke).toHaveBeenCalledWith("edit_track_metadata", {
        input: {
          libraryId: "lib-1",
          trackId: "trk-1",
          title: "Hotel California Live",
          artist: "Eagles",
          album: "Hotel California",
          trackNumber: 1,
          discNumber: 1,
          genre: "Rock",
          year: 1976,
        },
      }),
    );
    await waitFor(() =>
      expect(screen.getByText("Hotel California Live")).toBeDefined(),
    );
  });

  it("adds a track to an existing playlist", async () => {
    const user = userEvent.setup();
    mockedInvoke.mockImplementation((command: string) => {
      if (command === "list_tracks") return Promise.resolve(sampleTracks);
      if (command === "list_playlists") return Promise.resolve(samplePlaylists);
      if (command === "add_track_to_playlist") {
        return Promise.resolve({
          ...samplePlaylists[1],
          entries: [{ id: "entry-1", trackId: "trk-1", position: 0 }],
        });
      }
      return Promise.resolve(undefined);
    });
    render(
      <TrackList
        libraryId="lib-1"
        currentTrackId={null}
        isPlaying={false}
        onPlayTrack={() => {}}
        onToggleCurrentTrack={() => {}}
      />,
    );

    await waitFor(() => expect(screen.getByText("Hotel California")).toBeDefined());
    await user.click(
      screen.getByRole("button", {
        name: "Add Hotel California to playlist",
      }),
    );
    await user.selectOptions(screen.getByLabelText("Playlist"), "playlist-2");
    await user.click(screen.getByRole("button", { name: "Add to playlist" }));

    await waitFor(() =>
      expect(mockedInvoke).toHaveBeenCalledWith("add_track_to_playlist", {
        input: {
          playlistId: "playlist-2",
          trackId: "trk-1",
        },
      }),
    );
    expect(screen.getByRole("status").textContent).toContain(
      "Added Hotel California to Late Night.",
    );
  });

  it("shows an empty playlist state without adding the track", async () => {
    const user = userEvent.setup();
    mockedInvoke.mockImplementation((command: string) => {
      if (command === "list_tracks") return Promise.resolve(sampleTracks);
      if (command === "list_playlists") return Promise.resolve([]);
      return Promise.resolve(undefined);
    });
    render(
      <TrackList
        libraryId="lib-1"
        currentTrackId={null}
        isPlaying={false}
        onPlayTrack={() => {}}
        onToggleCurrentTrack={() => {}}
      />,
    );

    await waitFor(() => expect(screen.getByText("Hotel California")).toBeDefined());
    await user.click(
      screen.getByRole("button", {
        name: "Add Hotel California to playlist",
      }),
    );

    await waitFor(() =>
      expect(screen.getByText("No playlists yet.")).toBeDefined(),
    );
    expect(mockedInvoke).not.toHaveBeenCalledWith(
      "add_track_to_playlist",
      expect.anything(),
    );
  });

  it("keeps the playlist picker open and shows playlist load failures", async () => {
    const user = userEvent.setup();
    mockedInvoke.mockImplementation((command: string) => {
      if (command === "list_tracks") return Promise.resolve(sampleTracks);
      if (command === "list_playlists") {
        return Promise.reject(new Error("Could not load playlists"));
      }
      return Promise.resolve(undefined);
    });
    render(
      <TrackList
        libraryId="lib-1"
        currentTrackId={null}
        isPlaying={false}
        onPlayTrack={() => {}}
        onToggleCurrentTrack={() => {}}
      />,
    );

    await waitFor(() => expect(screen.getByText("Hotel California")).toBeDefined());
    await user.click(
      screen.getByRole("button", {
        name: "Add Hotel California to playlist",
      }),
    );

    await waitFor(() =>
      expect(screen.getByRole("alert").textContent).toContain(
        "Could not load playlists",
      ),
    );
    expect(screen.getByText("No playlists yet.")).toBeDefined();
  });

  it("keeps the playlist picker open and shows add failures", async () => {
    const user = userEvent.setup();
    mockedInvoke.mockImplementation((command: string) => {
      if (command === "list_tracks") return Promise.resolve(sampleTracks);
      if (command === "list_playlists") return Promise.resolve(samplePlaylists);
      if (command === "add_track_to_playlist") {
        return Promise.reject(new Error("Could not update playlist"));
      }
      return Promise.resolve(undefined);
    });
    render(
      <TrackList
        libraryId="lib-1"
        currentTrackId={null}
        isPlaying={false}
        onPlayTrack={() => {}}
        onToggleCurrentTrack={() => {}}
      />,
    );

    await waitFor(() => expect(screen.getByText("Hotel California")).toBeDefined());
    await user.click(
      screen.getByRole("button", {
        name: "Add Hotel California to playlist",
      }),
    );
    await user.click(screen.getByRole("button", { name: "Add to playlist" }));

    await waitFor(() =>
      expect(screen.getByRole("alert").textContent).toContain(
        "Could not update playlist",
      ),
    );
    expect(screen.getByLabelText("Playlist")).toBeDefined();
  });

  it("only offers clear override for tracks with library overrides", async () => {
    const user = userEvent.setup();
    mockedInvoke.mockResolvedValueOnce(sampleTracks);
    render(
      <TrackList
        libraryId="lib-1"
        currentTrackId={null}
        isPlaying={false}
        onPlayTrack={() => {}}
        onToggleCurrentTrack={() => {}}
      />,
    );

    await waitFor(() => expect(screen.getByText("Hotel California")).toBeDefined());
    await user.click(screen.getByRole("button", { name: /edit hotel california/i }));

    expect(screen.queryByRole("button", { name: "Clear override" })).toBeNull();
  });

  it("clears a metadata override and reloads file metadata", async () => {
    const user = userEvent.setup();
    mockedInvoke
      .mockResolvedValueOnce(sampleTracks)
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce([
        {
          ...sampleTracks[1],
          title: "Take It Easy From File",
          album: "Eagles",
          metadataOverridden: false,
        },
      ]);
    render(
      <TrackList
        libraryId="lib-1"
        currentTrackId={null}
        isPlaying={false}
        onPlayTrack={() => {}}
        onToggleCurrentTrack={() => {}}
      />,
    );

    await waitFor(() => expect(screen.getByText("Take It Easy")).toBeDefined());
    await user.click(screen.getByRole("button", { name: /edit take it easy/i }));
    await user.click(screen.getByRole("button", { name: "Clear override" }));

    await waitFor(() =>
      expect(mockedInvoke).toHaveBeenCalledWith(
        "clear_track_metadata_override",
        {
          input: {
            libraryId: "lib-1",
            trackId: "trk-2",
          },
        },
      ),
    );
    await waitFor(() =>
      expect(screen.getByText("Take It Easy From File")).toBeDefined(),
    );
    expect(screen.getByText("Eagles - Eagles")).toBeDefined();
  });

  it("keeps the editor open and shows save failures", async () => {
    const user = userEvent.setup();
    mockedInvoke
      .mockResolvedValueOnce(sampleTracks)
      .mockRejectedValueOnce(new Error("Could not write override"));
    render(
      <TrackList
        libraryId="lib-1"
        currentTrackId={null}
        isPlaying={false}
        onPlayTrack={() => {}}
        onToggleCurrentTrack={() => {}}
      />,
    );

    await waitFor(() => expect(screen.getByText("Hotel California")).toBeDefined());
    await user.click(screen.getByRole("button", { name: /edit hotel california/i }));
    await user.clear(screen.getByLabelText("Title"));
    await user.type(screen.getByLabelText("Title"), "Hotel California Live");
    await user.click(screen.getByRole("button", { name: "Save metadata" }));

    await waitFor(() =>
      expect(screen.getByRole("alert").textContent).toContain(
        "Could not write override",
      ),
    );
    expect((screen.getByLabelText("Title") as HTMLInputElement).value).toBe(
      "Hotel California Live",
    );
  });

  it("keeps the editor open and shows clear override failures", async () => {
    const user = userEvent.setup();
    mockedInvoke
      .mockResolvedValueOnce(sampleTracks)
      .mockRejectedValueOnce(new Error("Could not clear override"));
    render(
      <TrackList
        libraryId="lib-1"
        currentTrackId={null}
        isPlaying={false}
        onPlayTrack={() => {}}
        onToggleCurrentTrack={() => {}}
      />,
    );

    await waitFor(() => expect(screen.getByText("Take It Easy")).toBeDefined());
    await user.click(screen.getByRole("button", { name: /edit take it easy/i }));
    await user.click(screen.getByRole("button", { name: "Clear override" }));

    await waitFor(() =>
      expect(screen.getByRole("alert").textContent).toContain(
        "Could not clear override",
      ),
    );
    expect(screen.getByLabelText("Title")).toBeDefined();
  });

  it("shows the empty state when the library has no tracks", async () => {
    mockedInvoke.mockResolvedValueOnce([]);
    render(
      <TrackList
        libraryId="lib-1"
        currentTrackId={null}
        isPlaying={false}
        onPlayTrack={() => {}}
        onToggleCurrentTrack={() => {}}
      />,
    );

    await waitFor(() =>
      expect(screen.getByText(/no tracks yet/i)).toBeDefined(),
    );
  });

  it("calls onPlayTrack with the clicked track", async () => {
    const user = userEvent.setup();
    const onPlayTrack = vi.fn();
    const onToggleCurrentTrack = vi.fn();
    mockedInvoke.mockResolvedValueOnce(sampleTracks);
    render(
      <TrackList
        libraryId="lib-1"
        currentTrackId={null}
        isPlaying={false}
        onPlayTrack={onPlayTrack}
        onToggleCurrentTrack={onToggleCurrentTrack}
      />,
    );

    await waitFor(() => expect(screen.getByText("Hotel California")).toBeDefined());
    await user.click(screen.getByRole("button", { name: /play hotel california/i }));

    expect(onPlayTrack).toHaveBeenCalledWith(sampleTracks[0]);
    expect(onToggleCurrentTrack).not.toHaveBeenCalled();
  });

  it("toggles the current track instead of reloading it", async () => {
    const user = userEvent.setup();
    const onPlayTrack = vi.fn();
    const onToggleCurrentTrack = vi.fn();
    mockedInvoke.mockResolvedValueOnce(sampleTracks);
    render(
      <TrackList
        libraryId="lib-1"
        currentTrackId="trk-1"
        isPlaying
        onPlayTrack={onPlayTrack}
        onToggleCurrentTrack={onToggleCurrentTrack}
      />,
    );

    await waitFor(() => expect(screen.getByText("Hotel California")).toBeDefined());
    await user.click(screen.getByRole("button", { name: /pause hotel california/i }));

    expect(onToggleCurrentTrack).toHaveBeenCalledOnce();
    expect(onPlayTrack).not.toHaveBeenCalled();
  });

  it("marks the current track row as current", async () => {
    mockedInvoke.mockResolvedValueOnce(sampleTracks);
    render(
      <TrackList
        libraryId="lib-1"
        currentTrackId="trk-2"
        isPlaying={false}
        onPlayTrack={() => {}}
        onToggleCurrentTrack={() => {}}
      />,
    );

    await waitFor(() => expect(screen.getByText("Take It Easy")).toBeDefined());

    const row = screen.getByText("Take It Easy").closest("li")!;
    expect(row.className).toContain("track-list__row--current");
  });
});
