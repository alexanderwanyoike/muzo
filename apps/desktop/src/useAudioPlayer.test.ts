import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useAudioPlayer } from "./useAudioPlayer";
import type { TrackDto } from "./api";

const apiMocks = vi.hoisted(() => ({
  loadTrackAudioSource: vi.fn(),
}));

vi.mock("./api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./api")>()),
  loadTrackAudioSource: apiMocks.loadTrackAudioSource,
}));

type Listener = (event: unknown) => void;

class MockAudio {
  static instances: MockAudio[] = [];
  static nextPlayError: Error | null = null;

  src = "";
  currentTime = 0;
  duration = 0;
  volume = 1;
  paused = true;
  load = vi.fn();
  private listeners = new Map<string, Set<Listener>>();

  constructor() {
    MockAudio.instances.push(this);
  }

  play() {
    if (MockAudio.nextPlayError) {
      const error = MockAudio.nextPlayError;
      MockAudio.nextPlayError = null;
      return Promise.reject(error);
    }
    this.paused = false;
    this.listeners.get("play")?.forEach((fn) => fn({}));
    return Promise.resolve();
  }
  pause() {
    this.paused = true;
    this.listeners.get("pause")?.forEach((fn) => fn({}));
  }
  addEventListener(type: string, listener: Listener) {
    let set = this.listeners.get(type);
    if (!set) {
      set = new Set();
      this.listeners.set(type, set);
    }
    set.add(listener);
  }
  removeEventListener(type: string, listener: Listener) {
    this.listeners.get(type)?.delete(listener);
  }
  emit(type: string) {
    this.listeners.get(type)?.forEach((fn) => fn({}));
  }
}

beforeEach(() => {
  MockAudio.instances = [];
  MockAudio.nextPlayError = null;
  apiMocks.loadTrackAudioSource.mockReset();
  apiMocks.loadTrackAudioSource.mockResolvedValue({
    mimeType: "audio/mpeg",
    bytes: [1, 2, 3],
  });
  vi.stubGlobal("Audio", MockAudio);
  Object.defineProperty(URL, "createObjectURL", {
    configurable: true,
    value: vi.fn(() => "blob:track-audio"),
  });
  Object.defineProperty(URL, "revokeObjectURL", {
    configurable: true,
    value: vi.fn(),
  });
});

const track: TrackDto = {
  id: "trk-1",
  libraryId: "lib-1",
  title: "Hotel California",
  artist: "Eagles",
  durationSeconds: 391,
  filePath: "/music/Eagles/Hotel California.mp3",
};

describe("useAudioPlayer", () => {
  it("starts idle with no current track", () => {
    const { result } = renderHook(() => useAudioPlayer());
    expect(result.current.status).toBe("idle");
    expect(result.current.current).toBeNull();
  });

  it("loads a blob audio source and waits for the audio engine to start", async () => {
    const { result } = renderHook(() => useAudioPlayer());

    await act(async () => {
      result.current.play(track);
    });

    const audio = MockAudio.instances[0];
    expect(apiMocks.loadTrackAudioSource).toHaveBeenCalledWith({
      libraryId: "lib-1",
      trackId: "trk-1",
    });
    expect(URL.createObjectURL).toHaveBeenCalledWith(expect.any(Blob));
    expect(audio.src).toBe("blob:track-audio");
    expect(audio.load).toHaveBeenCalledOnce();
    expect(result.current.current).toEqual(track);
    expect(result.current.status).toBe("loading");

    act(() => {
      audio.emit("playing");
    });

    expect(result.current.status).toBe("playing");
  });

  it("toggles between playing and paused", async () => {
    const { result } = renderHook(() => useAudioPlayer());

    await act(async () => {
      result.current.play(track);
    });
    act(() => {
      MockAudio.instances[0].emit("playing");
    });
    expect(result.current.status).toBe("playing");

    act(() => {
      result.current.toggle();
    });
    expect(result.current.status).toBe("paused");

    act(() => {
      result.current.toggle();
    });
    expect(result.current.status).toBe("loading");

    act(() => {
      MockAudio.instances[0].emit("playing");
    });
    expect(result.current.status).toBe("playing");
  });

  it("updates position when seek is called", async () => {
    const { result } = renderHook(() => useAudioPlayer());

    await act(async () => {
      result.current.play(track);
    });
    act(() => {
      result.current.seek(120);
    });

    expect(result.current.positionSeconds).toBe(120);
  });

  it("clamps volume to the 0..1 range", () => {
    const { result } = renderHook(() => useAudioPlayer());

    act(() => {
      result.current.setVolume(2);
    });
    expect(result.current.volume).toBe(1);

    act(() => {
      result.current.setVolume(-1);
    });
    expect(result.current.volume).toBe(0);

    act(() => {
      result.current.setVolume(0.5);
    });
    expect(result.current.volume).toBe(0.5);
  });

  it("does nothing when toggle is called before any track is loaded", () => {
    const { result } = renderHook(() => useAudioPlayer());

    act(() => {
      result.current.toggle();
    });

    expect(result.current.status).toBe("idle");
    expect(result.current.current).toBeNull();
  });

  it("reports an error when the audio element fails to load the source", async () => {
    const { result } = renderHook(() => useAudioPlayer());

    await act(async () => {
      result.current.play(track);
    });
    act(() => {
      MockAudio.instances[0].emit("error");
    });

    expect(result.current.status).toBe("error");
    expect(result.current.playbackError).toMatch(/could not play/i);
  });

  it("includes the rejected play error details", async () => {
    MockAudio.nextPlayError = new DOMException("The element has no supported sources.", "NotSupportedError");
    const { result } = renderHook(() => useAudioPlayer());

    await act(async () => {
      result.current.play(track);
    });

    expect(result.current.status).toBe("error");
    expect(result.current.playbackError).toContain("NotSupportedError");
    expect(result.current.playbackError).toContain("The element has no supported sources.");
  });

  it("reports an error when the track audio source cannot be loaded", async () => {
    apiMocks.loadTrackAudioSource.mockRejectedValue(new Error("track file is missing"));
    const { result } = renderHook(() => useAudioPlayer());

    await act(async () => {
      result.current.play(track);
    });

    expect(result.current.status).toBe("error");
    expect(result.current.playbackError).toContain("Could not play this track");
    expect(result.current.playbackError).toContain("track file is missing");
  });
});
