import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import TrackList from "./TrackList";
import type { TrackDto } from "./api";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";

const mockedInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

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

beforeEach(() => {
  mockedInvoke.mockReset();
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
