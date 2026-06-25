import { describe, it, expect, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import NowPlayingBar from "./NowPlayingBar";
import type { TrackDto } from "./api";

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
  filePath: "/music/Eagles/Hotel California.mp3",
};

describe("NowPlayingBar", () => {
  it("renders an idle transport when there is no current track", () => {
    render(
      <NowPlayingBar
        current={null}
        status="idle"
        positionSeconds={0}
        durationSeconds={0}
        volume={1}
        playbackError={null}
        onToggle={() => {}}
        onSeek={() => {}}
        onSetVolume={() => {}}
      />,
    );
    expect(screen.getByText("Not Playing")).toBeDefined();
    expect(screen.getByRole("button", { name: "Play" }).hasAttribute("disabled")).toBe(true);
  });

  it("shows the track title and artist when a track is loaded", () => {
    render(
      <NowPlayingBar
        current={track}
        status="paused"
        positionSeconds={0}
        durationSeconds={391}
        volume={1}
        playbackError={null}
        onToggle={() => {}}
        onSeek={() => {}}
        onSetVolume={() => {}}
      />,
    );

    expect(screen.getByText("Hotel California")).toBeDefined();
    expect(screen.getByText("Eagles")).toBeDefined();
  });

  it("calls onToggle when the play/pause button is clicked", async () => {
    const user = userEvent.setup();
    const onToggle = vi.fn();

    render(
      <NowPlayingBar
        current={track}
        status="paused"
        positionSeconds={0}
        durationSeconds={391}
        volume={1}
        playbackError={null}
        onToggle={onToggle}
        onSeek={() => {}}
        onSetVolume={() => {}}
      />,
    );

    await user.click(screen.getByRole("button", { name: "Play" }));
    expect(onToggle).toHaveBeenCalledOnce();
  });

  it("shows Pause label when status is playing", () => {
    render(
      <NowPlayingBar
        current={track}
        status="playing"
        positionSeconds={120}
        durationSeconds={391}
        volume={1}
        playbackError={null}
        onToggle={() => {}}
        onSeek={() => {}}
        onSetVolume={() => {}}
      />,
    );

    expect(screen.getByRole("button", { name: "Pause" })).toBeDefined();
  });

  it("shows elapsed and remaining time", () => {
    render(
      <NowPlayingBar
        current={track}
        status="playing"
        positionSeconds={120}
        durationSeconds={391}
        volume={1}
        playbackError={null}
        onToggle={() => {}}
        onSeek={() => {}}
        onSetVolume={() => {}}
      />,
    );

    expect(screen.getByText("2:00")).toBeDefined();
    expect(screen.getByText("6:31")).toBeDefined();
  });

  it("seeks with a draggable progress slider", () => {
    const onSeek = vi.fn();

    render(
      <NowPlayingBar
        current={track}
        status="playing"
        positionSeconds={120}
        durationSeconds={391}
        volume={1}
        playbackError={null}
        onToggle={() => {}}
        onSeek={onSeek}
        onSetVolume={() => {}}
      />,
    );

    const slider = screen.getByRole("slider", { name: "Seek" });
    fireEvent.change(slider, { target: { value: "180" } });

    expect(onSeek).toHaveBeenCalledWith(180);
  });

  it("aligns the visual progress fill to the current position", () => {
    const { container } = render(
      <NowPlayingBar
        current={track}
        status="playing"
        positionSeconds={120}
        durationSeconds={240}
        volume={1}
        playbackError={null}
        onToggle={() => {}}
        onSeek={() => {}}
        onSetVolume={() => {}}
      />,
    );

    const fill = container.querySelector<HTMLElement>(".now-playing-bar__progress-fill");
    expect(fill?.style.transform).toBe("scaleX(0.5)");
  });

  it("shows playback errors", () => {
    render(
      <NowPlayingBar
        current={track}
        status="error"
        positionSeconds={0}
        durationSeconds={391}
        volume={1}
        playbackError="Could not play this track."
        onToggle={() => {}}
        onSeek={() => {}}
        onSetVolume={() => {}}
      />,
    );

    expect(screen.getByText("Could not play this track.")).toBeDefined();
  });
});
