import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useAudioPlayer } from "./useAudioPlayer";
import type { TrackDto } from "./api";

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: vi.fn((path: string) => `asset://${path}`),
}));

type Listener = (event: unknown) => void;

class MockAudio {
  src = "";
  currentTime = 0;
  duration = 0;
  volume = 1;
  paused = true;
  private listeners = new Map<string, Set<Listener>>();

  play() {
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
}

beforeEach(() => {
  vi.stubGlobal("Audio", MockAudio);
});

const track: TrackDto = {
  id: "trk-1",
  libraryId: "lib-1",
  title: "Hotel California",
  artist: "Eagles",
  durationSeconds: 391,
  filePath: "asset://localhost/music/Hotel%20California.mp3",
};

describe("useAudioPlayer", () => {
  it("starts idle with no current track", () => {
    const { result } = renderHook(() => useAudioPlayer());
    expect(result.current.status).toBe("idle");
    expect(result.current.current).toBeNull();
  });

  it("sets the current track and switches to playing when play is called", () => {
    const { result } = renderHook(() => useAudioPlayer());

    act(() => {
      result.current.play(track);
    });

    expect(result.current.current).toEqual(track);
    expect(result.current.status).toBe("playing");
  });

  it("toggles between playing and paused", () => {
    const { result } = renderHook(() => useAudioPlayer());

    act(() => {
      result.current.play(track);
    });
    expect(result.current.status).toBe("playing");

    act(() => {
      result.current.toggle();
    });
    expect(result.current.status).toBe("paused");

    act(() => {
      result.current.toggle();
    });
    expect(result.current.status).toBe("playing");
  });

  it("updates position when seek is called", () => {
    const { result } = renderHook(() => useAudioPlayer());

    act(() => {
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
});
