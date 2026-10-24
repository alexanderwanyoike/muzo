import { prepareTrackAudioSource, recordTrackPlay, type TrackDto } from "./api";
import type { PlayerStatus } from "./useAudioPlayer";

export interface AudioElementLike {
  src: string;
  currentTime: number;
  duration: number;
  volume: number;
  paused: boolean;
  load: () => void;
  play: () => Promise<void> | void;
  pause: () => void;
  addEventListener: (type: string, listener: (event: unknown) => void) => void;
  removeEventListener: (type: string, listener: (event: unknown) => void) => void;
}

export async function loadAndPlayTrack(
  audio: AudioElementLike,
  track: TrackDto,
  requestId: number,
  playRequestRef: React.MutableRefObject<number>,
  currentTrackIdRef: React.MutableRefObject<string | null>,
  setStatus: React.Dispatch<React.SetStateAction<PlayerStatus>>,
  setPlaybackError: React.Dispatch<React.SetStateAction<string | null>>,
  onTrackPlayRecorded?: (track: TrackDto) => void,
) {
  let source;
  try {
    source = await prepareTrackAudioSource({
      libraryId: track.libraryId,
      trackId: track.id,
    });
  } catch (err) {
    if (requestId !== playRequestRef.current) return;
    currentTrackIdRef.current = null;
    setStatus("error");
    setPlaybackError(formatPlaybackError("The audio source could not be loaded", err));
    return;
  }

  if (requestId !== playRequestRef.current) return;
  audio.src = source.url;
  audio.load();

  try {
    await Promise.resolve(audio.play());
  } catch (err) {
    if (requestId !== playRequestRef.current) return;
    setStatus("error");
    setPlaybackError(formatPlaybackError("The audio engine rejected playback", err));
    return;
  }
  if (requestId !== playRequestRef.current) return;
  try {
    await recordTrackPlay({
      libraryId: track.libraryId,
      trackId: track.id,
    });
  } catch {
    return;
  }
  if (requestId !== playRequestRef.current) return;
  onTrackPlayRecorded?.(track);
}

export function createAudioElement(): { audio: AudioElementLike; cleanup: () => void } {
  if (typeof Audio !== "undefined") {
    const audio = new Audio() as unknown as AudioElementLike;
    const maybeDomAudio = audio as Partial<HTMLAudioElement>;
    maybeDomAudio.preload = "auto";
    if (maybeDomAudio.style) {
      maybeDomAudio.style.display = "none";
    }
    if (typeof document !== "undefined" && maybeDomAudio instanceof Node) {
      document.body.appendChild(maybeDomAudio);
      return {
        audio,
        cleanup: () => maybeDomAudio.remove?.(),
      };
    }
    return { audio, cleanup: () => {} };
  }
  return { audio: createStubAudioElement(), cleanup: () => {} };
}

export function formatPlaybackError(reason: string, err: unknown): string {
  if (err instanceof Error) {
    return `Could not play this track. ${reason}: ${err.name}: ${err.message}`;
  }
  if (isErrorLike(err)) {
    return `Could not play this track. ${reason}: ${err.name}: ${err.message}`;
  }
  if (typeof err === "string" && err.trim() !== "") {
    return `Could not play this track. ${reason}: ${err}`;
  }
  return `Could not play this track. ${reason}.`;
}

function isErrorLike(err: unknown): err is { name: string; message: string } {
  if (!err || typeof err !== "object") return false;
  return "name" in err && "message" in err && typeof err.name === "string" && typeof err.message === "string";
}

function createStubAudioElement(): AudioElementLike {
  const listeners = new Map<string, Set<(event: unknown) => void>>();
  const state = {
    src: "",
    currentTime: 0,
    duration: 0,
    volume: 1,
    paused: true,
  };
  return {
    get src() {
      return state.src;
    },
    set src(value: string) {
      state.src = value;
    },
    get currentTime() {
      return state.currentTime;
    },
    set currentTime(value: number) {
      state.currentTime = value;
    },
    get duration() {
      return state.duration;
    },
    set duration(value: number) {
      state.duration = value;
    },
    get volume() {
      return state.volume;
    },
    set volume(value: number) {
      state.volume = value;
    },
    get paused() {
      return state.paused;
    },
    load: () => {},
    play: () => {
      state.paused = false;
      listeners.get("playing")?.forEach((fn) => fn({}));
      return Promise.resolve();
    },
    pause: () => {
      state.paused = true;
      listeners.get("pause")?.forEach((fn) => fn({}));
    },
    addEventListener: (type, listener) => {
      let set = listeners.get(type);
      if (!set) {
        set = new Set();
        listeners.set(type, set);
      }
      set.add(listener);
    },
    removeEventListener: (type, listener) => {
      listeners.get(type)?.delete(listener);
    },
  };
}
