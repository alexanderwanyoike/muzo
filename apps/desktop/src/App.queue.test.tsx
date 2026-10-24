import { afterEach, expect, it, vi } from "vitest";
import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "./App";
import { installTestRuntime, removeTestRuntime } from "./test-runtime";

class TestAudio extends EventTarget {
  static current: TestAudio;

  src = "";

  currentTime = 0;

  duration = 120;

  volume = 1;

  paused = true;

  constructor() { super(); TestAudio.current = this; }

  load() {}

  async play() { this.paused = false; this.dispatchEvent(new Event("playing")); }

  pause() { this.paused = true; this.dispatchEvent(new Event("pause")); }
}

afterEach(() => { cleanup(); removeTestRuntime(); vi.unstubAllGlobals(); });
it("plays a playlist, advances at track end, and navigates through the now-playing bar", async () => {
  vi.stubGlobal("Audio", TestAudio);
  const invoke = vi.fn().mockImplementation(async (command: string, args?: { input?: { trackId?: string } }) => {
    if (command === "list_playlists") return [{ id: "p", name: "Evening", entries: [
      { id: "a", trackId: "first", trackTitle: "First", trackLibraryId: "library", position: 0 },
      { id: "b", trackId: "second", trackTitle: "Second", trackLibraryId: "library", position: 1 },
    ] }];
    if (command === "prepare_track_audio_source") return { url: `source-${args?.input?.trackId}`, mimeType: "audio/mpeg" };
    return [];
  });
  installTestRuntime({ invoke, openDirectory: async () => null });
  const user = userEvent.setup();
  render(<App />);
  await user.click(screen.getByRole("button", { name: "Playlists" }));
  await user.click(await screen.findByRole("button", { name: "Play Evening" }));
  await waitFor(() => expect(invoke).toHaveBeenCalledWith("prepare_track_audio_source", { input: { libraryId: "library", trackId: "first" } }));
  expect(screen.getByRole("button", { name: "Next track" }).hasAttribute("disabled")).toBe(false);
  await act(async () => { TestAudio.current.dispatchEvent(new Event("ended")); });
  await waitFor(() => expect(invoke).toHaveBeenCalledWith("prepare_track_audio_source", { input: { libraryId: "library", trackId: "second" } }));
  expect(screen.getByRole("button", { name: "Next track" }).hasAttribute("disabled")).toBe(true);
  await user.click(screen.getByRole("button", { name: "Previous track" }));
  await waitFor(() => expect(TestAudio.current.src).toBe("source-first"));
  expect(screen.getByRole("button", { name: "Previous track" }).hasAttribute("disabled")).toBe(true);
});

function installQueueAndSongs() {
  vi.stubGlobal("Audio", TestAudio);
  const invoke = vi.fn().mockImplementation(async (command: string, args?: { input?: { trackId?: string } }) => {
    if (command === "list_libraries") return [{ id: "library", name: "Music", kind: "filesystem", location: "/Music" }];
    if (command === "list_tracks") return [{ id: "first", libraryId: "library", title: "First", artist: "Artist", album: null,
      trackNumber: null, discNumber: null, genre: null, year: null, metadataOverridden: false, durationSeconds: 120, filePath: "/Music/first.mp3" }];
    if (command === "list_playlists") return [{ id: "p", name: "Evening", entries: [
      { id: "a", trackId: "first", trackTitle: "First", trackLibraryId: "library", position: 0 },
      { id: "b", trackId: "second", trackTitle: "Second", trackLibraryId: "library", position: 1 },
    ] }];
    if (command === "prepare_track_audio_source") return { url: `source-${args?.input?.trackId}`, mimeType: "audio/mpeg" };
    return [];
  });
  installTestRuntime({ invoke, openDirectory: async () => null });
  return invoke;
}

it("starts a playlist queue when its first song is already playing directly", async () => {
  installQueueAndSongs();
  const user = userEvent.setup();
  render(<App />);
  await user.click(await screen.findByRole("button", { name: "Show tracks for Music" }));
  await user.click(await screen.findByRole("button", { name: "Play First" }));
  await waitFor(() => expect(TestAudio.current.paused).toBe(false));
  await user.click(screen.getByRole("button", { name: "Playlists" }));
  await user.click(await screen.findByRole("button", { name: /^(Play|Pause) First from Evening$/ }));
  expect(screen.getByRole("button", { name: "Next track" }).hasAttribute("disabled")).toBe(false);
  await act(async () => { TestAudio.current.dispatchEvent(new Event("ended")); });
  await waitFor(() => expect(TestAudio.current.src).toBe("source-second"));
});

it("clears the playlist queue when selecting its current song directly from Songs", async () => {
  const invoke = installQueueAndSongs();
  const user = userEvent.setup();
  render(<App />);
  await user.click(screen.getByRole("button", { name: "Playlists" }));
  await user.click(await screen.findByRole("button", { name: "Play Evening" }));
  await waitFor(() => expect(TestAudio.current.paused).toBe(false));
  await user.click(await screen.findByRole("button", { name: "Show tracks for Music" }));
  await user.click(await screen.findByRole("button", { name: "Pause First" }));
  expect(screen.getByRole("button", { name: "Next track" }).hasAttribute("disabled")).toBe(true);
  await act(async () => { TestAudio.current.dispatchEvent(new Event("ended")); });
  expect(invoke).not.toHaveBeenCalledWith("prepare_track_audio_source", { input: { libraryId: "library", trackId: "second" } });
});
