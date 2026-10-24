import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useAudioPlayer } from "./useAudioPlayer";
import type { TrackDto } from "./api";

const apiMocks = vi.hoisted(() => ({
  prepareTrackAudioSource: vi.fn(),
  recordTrackPlay: vi.fn(),
}));

vi.mock("./api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./api")>()),
  prepareTrackAudioSource: apiMocks.prepareTrackAudioSource,
  recordTrackPlay: apiMocks.recordTrackPlay,
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
  apiMocks.prepareTrackAudioSource.mockReset();
  apiMocks.recordTrackPlay.mockReset();
  apiMocks.prepareTrackAudioSource.mockResolvedValue({
    mimeType: "audio/mpeg",
    url: "http://127.0.0.1:49152/audio/trk-1",
  });
  apiMocks.recordTrackPlay.mockResolvedValue(undefined);
  vi.stubGlobal("Audio", MockAudio);
});

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

describe("useAudioPlayer", () => {
  it("starts idle with no current track", () => {
    const { result } = renderHook(() => useAudioPlayer());
    expect(result.current.status).toBe("idle");
    expect(result.current.current).toBeNull();
  });

  it("loads a streaming audio source and waits for the audio engine to start", async () => {
    const { result } = renderHook(() => useAudioPlayer());

    await act(async () => {
      result.current.play(track);
    });

    const audio = MockAudio.instances[0];
    expect(apiMocks.prepareTrackAudioSource).toHaveBeenCalledWith({
      libraryId: "lib-1",
      trackId: "trk-1",
    });
    expect(audio.src).toBe("http://127.0.0.1:49152/audio/trk-1");
    expect(audio.load).toHaveBeenCalledOnce();
    expect(result.current.current).toEqual(track);
    expect(result.current.status).toBe("loading");

    act(() => {
      audio.emit("playing");
    });

    expect(result.current.status).toBe("playing");
  });

  it("records one play when a new track starts successfully", async () => {
    const { result } = renderHook(() => useAudioPlayer());

    await act(async () => {
      result.current.play(track);
    });

    expect(apiMocks.recordTrackPlay).toHaveBeenCalledOnce();
    expect(apiMocks.recordTrackPlay).toHaveBeenCalledWith({
      libraryId: "lib-1",
      trackId: "trk-1",
    });
  });

  it("notifies after a new track play is recorded", async () => {
    const onTrackPlayRecorded = vi.fn();
    const { result } = renderHook(() =>
      useAudioPlayer({ onTrackPlayRecorded }),
    );

    await act(async () => {
      result.current.play(track);
    });

    expect(onTrackPlayRecorded).toHaveBeenCalledWith(track);
  });

  it("does not reload the current track when play is called again", async () => {
    const { result } = renderHook(() => useAudioPlayer());

    await act(async () => {
      result.current.play(track);
    });
    act(() => {
      MockAudio.instances[0].emit("playing");
      result.current.seek(42);
    });

    await act(async () => {
      result.current.play(track);
    });

    const audio = MockAudio.instances[0];
    expect(apiMocks.prepareTrackAudioSource).toHaveBeenCalledOnce();
    expect(apiMocks.recordTrackPlay).toHaveBeenCalledOnce();
    expect(audio.load).toHaveBeenCalledOnce();
    expect(result.current.positionSeconds).toBe(42);
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
    MockAudio.nextPlayError = new DOMException(
      "The element has no supported sources.",
      "NotSupportedError",
    );
    const { result } = renderHook(() => useAudioPlayer());

    await act(async () => {
      result.current.play(track);
    });

    expect(result.current.status).toBe("error");
    expect(result.current.playbackError).toContain("NotSupportedError");
    expect(result.current.playbackError).toContain(
      "The element has no supported sources.",
    );
    expect(apiMocks.recordTrackPlay).not.toHaveBeenCalled();
  });

  it("reports an error when the track audio source cannot be loaded", async () => {
    apiMocks.prepareTrackAudioSource.mockRejectedValue(
      new Error("track file is missing"),
    );
    const { result } = renderHook(() => useAudioPlayer());

    await act(async () => {
      result.current.play(track);
    });

    expect(result.current.status).toBe("error");
    expect(result.current.playbackError).toContain("Could not play this track");
    expect(result.current.playbackError).toContain("track file is missing");
    expect(apiMocks.recordTrackPlay).not.toHaveBeenCalled();
  });
  it("advances through duplicate playlist entries and stops at the end of the queue", async () => {
    const { result } = renderHook(() => useAudioPlayer());
    const queue = [
      { entryId: "first", track },
      { entryId: "repeat", track },
    ];
    await act(async () => {
      result.current.playQueue(queue);
    });
    expect(result.current.currentQueueEntryId).toBe("first");
    expect(result.current.hasNext).toBe(true);
    await act(async () => {
      MockAudio.instances[0].emit("ended");
    });
    expect(result.current.currentQueueEntryId).toBe("repeat");
    expect(apiMocks.prepareTrackAudioSource).toHaveBeenCalledTimes(2);
    expect(apiMocks.recordTrackPlay).toHaveBeenCalledTimes(2);
    expect(result.current.hasNext).toBe(false);
    act(() => {
      MockAudio.instances[0].emit("ended");
    });
    expect(result.current.status).toBe("paused");
    expect(apiMocks.prepareTrackAudioSource).toHaveBeenCalledTimes(2);
  });

  it("supports previous and next entries, and clears the queue when a song is selected directly", async () => {
    const { result } = renderHook(() => useAudioPlayer());
    const second = { ...track, id: "second" };
    await act(async () => {
      result.current.playQueue([
        { entryId: "a", track },
        { entryId: "b", track: second },
      ]);
    });
    expect(result.current.hasPrevious).toBe(false);
    await act(async () => {
      result.current.next();
    });
    expect(result.current.current?.id).toBe("second");
    expect(result.current.hasPrevious).toBe(true);
    await act(async () => {
      result.current.previous();
    });
    expect(result.current.currentQueueEntryId).toBe("a");
    await act(async () => {
      result.current.play({ ...track, id: "direct" });
    });
    expect(result.current.currentQueueEntryId).toBeNull();
    expect(result.current.hasNext).toBe(false);
    act(() => {
      MockAudio.instances[0].emit("ended");
    });
    expect(result.current.current?.id).toBe("direct");
  });

  it("pauses the old source while the next queued track is being prepared", async () => {
    const { result } = renderHook(() => useAudioPlayer());
    await act(async () => {
      result.current.playQueue([
        { entryId: "a", track },
        { entryId: "b", track: { ...track, id: "next" } },
      ]);
    });
    expect(MockAudio.instances[0].paused).toBe(false);
    let finish: (source: { url: string; mimeType: string }) => void = () => {};
    apiMocks.prepareTrackAudioSource.mockReturnValueOnce(
      new Promise((resolve) => {
        finish = resolve;
      }),
    );
    act(() => {
      result.current.next();
    });
    expect(MockAudio.instances[0].paused).toBe(true);
    await act(async () => {
      finish({ url: "next-source", mimeType: "audio/mpeg" });
    });
    expect(MockAudio.instances[0].src).toBe("next-source");
  });

  it("cancels pending playback and releases audio when the player unmounts", async () => {
    let finish: (source: { url: string; mimeType: string }) => void = () => {};
    apiMocks.prepareTrackAudioSource.mockReturnValueOnce(
      new Promise((resolve) => {
        finish = resolve;
      }),
    );
    const { result, unmount } = renderHook(() => useAudioPlayer());
    act(() => {
      result.current.play(track);
    });
    const audio = MockAudio.instances[0];
    unmount();
    await act(async () => {
      finish({ url: "late-source", mimeType: "audio/mpeg" });
    });
    expect(apiMocks.recordTrackPlay).not.toHaveBeenCalled();
    expect(audio.paused).toBe(true);
    expect(audio.src).toBe("");
  });

  it("does not resume a stale source while the next queue entry is loading", async () => {
    const { result } = renderHook(() => useAudioPlayer());
    await act(async () => {
      result.current.playQueue([
        { entryId: "a", track },
        { entryId: "b", track: { ...track, id: "second" } },
      ]);
    });
    let finish: (source: { url: string; mimeType: string }) => void = () => {};
    apiMocks.prepareTrackAudioSource.mockReturnValueOnce(
      new Promise((resolve) => {
        finish = resolve;
      }),
    );
    act(() => {
      result.current.next();
    });
    await act(async () => {
      result.current.toggle();
    });
    expect(MockAudio.instances[0].paused).toBe(true);
    await act(async () => {
      finish({ url: "second-source", mimeType: "audio/mpeg" });
    });
  });

  it("stops on a failed queued source and lets the user skip it without recording a play", async () => {
    const { result } = renderHook(() => useAudioPlayer());
    await act(async () => {
      result.current.playQueue([
        { entryId: "a", track },
        { entryId: "b", track: { ...track, id: "missing" } },
        { entryId: "c", track: { ...track, id: "last" } },
      ]);
    });
    apiMocks.prepareTrackAudioSource.mockRejectedValueOnce(
      new Error("Download failed"),
    );
    await act(async () => {
      MockAudio.instances[0].emit("ended");
    });
    expect(result.current.status).toBe("error");
    expect(result.current.currentQueueEntryId).toBe("b");
    expect(apiMocks.recordTrackPlay).toHaveBeenCalledTimes(1);
    await act(async () => {
      result.current.next();
    });
    expect(result.current.currentQueueEntryId).toBe("c");
    expect(apiMocks.recordTrackPlay).toHaveBeenCalledTimes(2);
  });
});

it("keeps old audio paused when a delayed pause event arrives during the next track's preparation", async () => {
  const { result } = renderHook(() => useAudioPlayer());
  const nextTrack = { ...track, id: "trk-2", title: "Next" };
  await act(async () =>
    result.current.playQueue([
      { entryId: "first", track },
      { entryId: "next", track: nextTrack },
    ]),
  );
  const audio = MockAudio.instances[0];
  act(() => audio.emit("playing"));
  const play = vi.spyOn(audio, "play");
  vi.spyOn(audio, "pause").mockImplementation(() => {
    audio.paused = true;
  });
  let finish: (source: { url: string; mimeType: string }) => void = () => {};
  apiMocks.prepareTrackAudioSource.mockReturnValueOnce(
    new Promise((resolve) => {
      finish = resolve;
    }),
  );
  act(() => result.current.next());
  expect(result.current.status).toBe("loading");
  act(() => audio.emit("pause"));
  act(() => result.current.toggle());
  expect(result.current.status).toBe("loading");
  expect(audio.paused).toBe(true);
  expect(play).not.toHaveBeenCalled();
  expect(apiMocks.recordTrackPlay).toHaveBeenCalledTimes(1);
  await act(async () => finish({ url: "source-next", mimeType: "audio/mpeg" }));
  expect(audio.src).toBe("source-next");
  expect(play).toHaveBeenCalledTimes(1);
  expect(apiMocks.recordTrackPlay).toHaveBeenCalledTimes(2);
});

it.each(["preparation", "engine"] as const)(
  "retries the selected queue track after %s failure instead of resuming stale audio",
  async (failureStage) => {
    const { result } = renderHook(() => useAudioPlayer());
    const nextTrack = { ...track, id: "next-track", title: "Next" };
    await act(async () =>
      result.current.playQueue([
        { entryId: "first", track },
        { entryId: "next", track: nextTrack },
      ]),
    );
    const audio = MockAudio.instances[0];
    act(() => audio.emit("playing"));
    if (failureStage === "preparation") {
      apiMocks.prepareTrackAudioSource.mockRejectedValueOnce(
        new Error("download failed"),
      );
    } else {
      apiMocks.prepareTrackAudioSource.mockResolvedValueOnce({
        url: "source-next",
        mimeType: "audio/mpeg",
      });
      MockAudio.nextPlayError = new Error("source failed");
    }
    await act(async () => result.current.next());
    expect(result.current.status).toBe("error");
    const play = vi.spyOn(audio, "play");
    let finish: (source: { url: string; mimeType: string }) => void = () => {};
    apiMocks.prepareTrackAudioSource.mockReturnValueOnce(
      new Promise((resolve) => {
        finish = resolve;
      }),
    );
    act(() => result.current.toggle());
    expect(apiMocks.prepareTrackAudioSource).toHaveBeenCalledTimes(3);
    expect(apiMocks.prepareTrackAudioSource).toHaveBeenLastCalledWith({
      libraryId: nextTrack.libraryId,
      trackId: nextTrack.id,
    });
    expect(result.current.status).toBe("loading");
    expect(result.current.currentQueueEntryId).toBe("next");
    expect(audio.paused).toBe(true);
    expect(play).not.toHaveBeenCalled();
    await act(async () =>
      finish({ url: "source-next-retry", mimeType: "audio/mpeg" }),
    );
    expect(audio.src).toBe("source-next-retry");
    expect(play).toHaveBeenCalledTimes(1);
    expect(apiMocks.recordTrackPlay).toHaveBeenCalledTimes(2);
    expect(apiMocks.recordTrackPlay).toHaveBeenLastCalledWith({
      libraryId: nextTrack.libraryId,
      trackId: nextTrack.id,
    });
  },
);

it("ignores a stale playing event from the paused source while the next track prepares", async () => {
  const { result } = renderHook(() => useAudioPlayer());
  const nextTrack = { ...track, id: "next-track" };
  await act(async () =>
    result.current.playQueue([
      { entryId: "a", track },
      { entryId: "b", track: nextTrack },
    ]),
  );
  const audio = MockAudio.instances[0];
  act(() => audio.emit("playing"));
  let finish: (source: { url: string; mimeType: string }) => void = () => {};
  apiMocks.prepareTrackAudioSource.mockReturnValueOnce(
    new Promise((resolve) => {
      finish = resolve;
    }),
  );
  act(() => result.current.next());
  const play = vi.spyOn(audio, "play");
  act(() => audio.emit("playing"));
  expect(result.current.status).toBe("loading");
  act(() => result.current.toggle());
  expect(play).not.toHaveBeenCalled();
  expect(audio.paused).toBe(true);
  await act(async () => finish({ url: "source-next", mimeType: "audio/mpeg" }));
  act(() => audio.emit("playing"));
  expect(result.current.status).toBe("playing");
});

it("prepares a fresh source after a media error while preserving the queue", async () => {
  const { result } = renderHook(() => useAudioPlayer());
  await act(async () =>
    result.current.playQueue([
      { entryId: "a", track },
      { entryId: "b", track: { ...track, id: "next" } },
    ]),
  );
  const audio = MockAudio.instances[0];
  act(() => {
    audio.emit("playing");
    audio.pause();
    audio.emit("error");
  });
  expect(result.current.status).toBe("error");
  let finish: (source: { url: string; mimeType: string }) => void = () => {};
  apiMocks.prepareTrackAudioSource.mockReturnValueOnce(
    new Promise((resolve) => {
      finish = resolve;
    }),
  );
  const play = vi.spyOn(audio, "play");
  act(() => result.current.toggle());
  expect(apiMocks.prepareTrackAudioSource).toHaveBeenCalledTimes(2);
  expect(result.current.status).toBe("loading");
  expect(result.current.currentQueueEntryId).toBe("a");
  expect(result.current.hasNext).toBe(true);
  expect(play).not.toHaveBeenCalled();
  await act(async () =>
    finish({ url: "source-retry", mimeType: "audio/mpeg" }),
  );
  expect(audio.src).toBe("source-retry");
  expect(apiMocks.recordTrackPlay).toHaveBeenCalledTimes(2);
});

it.each(["toggle", "select-current"] as const)(
  "prepares a fresh source after a rejected %s resume",
  async (resumeControl) => {
    const { result } = renderHook(() => useAudioPlayer());
    await act(async () => result.current.play(track));
    const audio = MockAudio.instances[0];
    act(() => audio.emit("playing"));
    act(() => result.current.toggle());
    MockAudio.nextPlayError = new Error("resume rejected");
    await act(async () => {
      if (resumeControl === "toggle") result.current.toggle();
      else result.current.play(track);
    });
    expect(result.current.status).toBe("error");
    let finish: (source: { url: string; mimeType: string }) => void = () => {};
    apiMocks.prepareTrackAudioSource.mockReturnValueOnce(
      new Promise((resolve) => {
        finish = resolve;
      }),
    );
    const play = vi.spyOn(audio, "play");
    act(() => result.current.toggle());
    expect(apiMocks.prepareTrackAudioSource).toHaveBeenCalledTimes(2);
    expect(result.current.status).toBe("loading");
    expect(play).not.toHaveBeenCalled();
    await act(async () =>
      finish({ url: "fresh-source", mimeType: "audio/mpeg" }),
    );
    expect(audio.src).toBe("fresh-source");
    expect(apiMocks.recordTrackPlay).toHaveBeenCalledTimes(2);
  },
);

it.each(["toggle", "select-current"] as const)(
  "ignores a late rejected %s resume after another track starts",
  async (resumeControl) => {
    const { result } = renderHook(() => useAudioPlayer());
    await act(async () => result.current.play(track));
    const audio = MockAudio.instances[0];
    act(() => audio.emit("playing"));
    act(() => result.current.toggle());
    let rejectResume: (error: Error) => void = () => {};
    const pendingResume = new Promise<void>((_resolve, reject) => {
      rejectResume = reject;
    });
    vi.spyOn(audio, "play").mockImplementationOnce(() => pendingResume);
    act(() => {
      if (resumeControl === "toggle") result.current.toggle();
      else result.current.play(track);
    });
    const nextTrack = { ...track, id: "next-track" };
    await act(async () => result.current.play(nextTrack));
    act(() => audio.emit("playing"));
    await act(async () => rejectResume(new Error("old resume failed")));
    expect(result.current.current?.id).toBe(nextTrack.id);
    expect(result.current.status).toBe("playing");
    expect(result.current.playbackError).toBeNull();
    act(() => result.current.toggle());
    await act(async () => result.current.toggle());
    expect(apiMocks.prepareTrackAudioSource).toHaveBeenCalledTimes(2);
  },
);

it("stops and invalidates pending playback when its source is removed", async () => {
  let prepare: ((value: { mimeType: string; url: string }) => void) | undefined;
  apiMocks.prepareTrackAudioSource.mockImplementationOnce(() => new Promise(resolve => { prepare = resolve; }));
  const { result } = renderHook(() => useAudioPlayer());
  act(() => result.current.play(track));
  act(() => result.current.removeSource(track.libraryId));
  await act(async () => { prepare?.({ mimeType: "audio/mpeg", url: "http://127.0.0.1/stale" }); });
  const audio = MockAudio.instances[0];
  audio.emit("pause");
  audio.emit("error");
  expect(result.current.current).toBeNull();
  expect(result.current.status).toBe("idle");
  expect(audio.src).toBe("");
  expect(audio.paused).toBe(true);
  expect(apiMocks.recordTrackPlay).not.toHaveBeenCalled();
});

it("removes unavailable queued entries without interrupting another source", async () => {
  const kept = { ...track, id: "kept", libraryId: "other" };
  const nextKept = { ...kept, id: "next-kept" };
  const { result } = renderHook(() => useAudioPlayer());
  await act(async () => result.current.playQueue([
    { entryId: "gone-before", track },
    { entryId: "current", track: kept },
    { entryId: "gone-after", track },
    { entryId: "next", track: nextKept },
  ], 1));
  const audio = MockAudio.instances[0];
  const originalSource = audio.src;
  act(() => result.current.removeSource("lib-1"));
  expect(result.current.current?.id).toBe("kept");
  expect(result.current.currentQueueEntryId).toBe("current");
  expect(result.current.hasPrevious).toBe(false);
  expect(result.current.hasNext).toBe(true);
  expect(audio.src).toBe(originalSource);
  await act(async () => result.current.next());
  expect(result.current.current?.id).toBe("next-kept");
});
