import { afterEach, expect, it, vi } from "vitest";
import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "./App";
import { installTestRuntime, removeTestRuntime } from "./test-runtime";

class TestAudio extends EventTarget {
  static instances: TestAudio[] = [];
  src = "";
  currentTime = 0;
  duration = 240;
  volume = 1;
  paused = true;
  constructor() {
    super();
    TestAudio.instances.push(this);
  }
  load() {}
  async play() {
    this.paused = false;
    this.dispatchEvent(new Event("playing"));
  }
  pause() {
    this.paused = true;
    this.dispatchEvent(new Event("pause"));
  }
}

function installMusic() {
  vi.stubGlobal("Audio", TestAudio);
  const track = {
    id: "first",
    libraryId: "library",
    title: "First light",
    artist: "Linden",
    album: "North Window",
    trackNumber: 1,
    discNumber: 1,
    genre: null,
    year: null,
    metadataOverridden: false,
    durationSeconds: 240,
    filePath: "/Music/first.flac",
  };
  const invoke = vi.fn().mockImplementation(async (command: string) => {
    if (command === "list_libraries")
      return [
        {
          id: "library",
          name: "Music",
          kind: "filesystem",
          location: "/Music",
        },
      ];
    if (command === "list_tracks") return [track];
    if (command === "prepare_track_audio_source")
      return { url: "source-first", mimeType: "audio/flac" };
    if (command === "get_track_artwork") return { ok: true, value: null };
    return [];
  });
  const setPlayerWindowMode = vi.fn(async () => ({
    ok: true as const,
    value: undefined,
  }));
  installTestRuntime({
    invoke,
    openDirectory: async () => null,
    setPlayerWindowMode,
  });
  return { invoke, setPlayerWindowMode };
}

afterEach(() => {
  cleanup();
  removeTestRuntime();
  vi.unstubAllGlobals();
  TestAudio.instances = [];
});

it("switches to the mini player and back without restarting playback or losing the source view", async () => {
  const { invoke, setPlayerWindowMode } = installMusic();
  const user = userEvent.setup();
  render(<App />);
  await user.click(
    await screen.findByRole("button", { name: "Show tracks for Music" }),
  );
  await user.click(
    await screen.findByRole("button", { name: "Play First light" }),
  );
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Pause" })).toBeDefined(),
  );
  const audio = TestAudio.instances[0];
  await act(async () => {
    audio.currentTime = 74;
    audio.dispatchEvent(new Event("timeupdate"));
  });
  await user.click(screen.getByRole("button", { name: "Mini player" }));
  expect(
    await screen.findByRole("region", { name: "Mini player" }),
  ).toBeDefined();
  expect(setPlayerWindowMode).toHaveBeenCalledWith("compact");
  expect(
    screen.getByRole("slider", { name: "Seek" }).getAttribute("value"),
  ).toBe("74");
  expect(audio.paused).toBe(false);
  expect(TestAudio.instances).toHaveLength(1);
  expect(
    invoke.mock.calls.filter(
      ([command]) => command === "prepare_track_audio_source",
    ),
  ).toHaveLength(1);
  await user.click(screen.getByRole("button", { name: "Return to library" }));
  expect(setPlayerWindowMode).toHaveBeenLastCalledWith("library");
  expect(screen.getByRole("heading", { name: "Music" })).toBeDefined();
  expect(audio.currentTime).toBe(74);
  expect(audio.paused).toBe(false);
});
