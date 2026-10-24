import { afterEach, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "./App";
import { installTestRuntime, removeTestRuntime } from "./test-runtime";

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  removeTestRuntime();
});

it("shows embedded artwork in Songs, the transport and the large listening sleeve", async () => {
  vi.spyOn(HTMLMediaElement.prototype, "play").mockResolvedValue(undefined);
  vi.spyOn(HTMLMediaElement.prototype, "pause").mockImplementation(() => {});
  vi.spyOn(HTMLMediaElement.prototype, "load").mockImplementation(() => {});
  const track = {
    id: "track",
    libraryId: "source",
    title: "Song",
    artist: "Artist",
    album: "Evening",
    trackNumber: 1,
    discNumber: 1,
    year: null,
    genre: null,
    metadataOverridden: false,
    durationSeconds: 120,
    filePath: "/music/song.flac",
  };
  const invoke = vi.fn().mockImplementation(async (command: string) => {
    if (command === "list_libraries")
      return [
        { id: "source", name: "Music", kind: "filesystem", location: "/music" },
      ];
    if (command === "list_tracks") return [track];
    if (command === "get_track_artwork")
      return { ok: true, value: "data:image/png;base64,cover" };
    if (command === "prepare_track_audio_source")
      return { mimeType: "audio/flac", url: "http://127.0.0.1/audio" };
    return [];
  });
  installTestRuntime({ invoke, openDirectory: vi.fn() });
  render(<App />);
  await userEvent.click(
    await screen.findByRole("button", { name: "Show tracks for Music" }),
  );
  expect(
    await screen.findByRole("img", { name: "Artwork for Evening" }),
  ).toBeDefined();
  await userEvent.click(screen.getByRole("button", { name: "Play Song" }));
  await waitFor(() =>
    expect(
      screen.getAllByRole("img", { name: "Artwork for Evening" }),
    ).toHaveLength(3),
  );
});
