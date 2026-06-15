import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { prepareTrackAudioSource, type TrackDto } from "./api";

export type PlayerStatus = "idle" | "loading" | "playing" | "paused" | "error";

export interface AudioPlayerState {
  current: TrackDto | null;
  status: PlayerStatus;
  positionSeconds: number;
  durationSeconds: number;
  volume: number;
  playbackError: string | null;
}

export interface AudioPlayer extends AudioPlayerState {
  play: (track: TrackDto) => void;
  toggle: () => void;
  seek: (seconds: number) => void;
  setVolume: (volume: number) => void;
}

interface AudioElementLike {
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

export function useAudioPlayer(): AudioPlayer {
  const audioRef = useRef<AudioElementLike | null>(null);
  const playRequestRef = useRef(0);
  const currentTrackIdRef = useRef<string | null>(null);
  const statusRef = useRef<PlayerStatus>("idle");
  const [current, setCurrent] = useState<TrackDto | null>(null);
  const [status, setStatus] = useState<PlayerStatus>("idle");
  const [positionSeconds, setPositionSeconds] = useState(0);
  const [durationSeconds, setDurationSeconds] = useState(0);
  const [volume, setVolumeState] = useState(1);
  const [playbackError, setPlaybackError] = useState<string | null>(null);

  useEffect(() => {
    statusRef.current = status;
  }, [status]);

  useEffect(() => {
    const { audio, cleanup } = createAudioElement();
    audioRef.current = audio;

    const onTimeUpdate = () => setPositionSeconds(audio.currentTime);
    const onDurationChange = () => setDurationSeconds(audio.duration || 0);
    const onEnded = () => setStatus("paused");
    const onPlaying = () => {
      setPlaybackError(null);
      setStatus("playing");
    };
    const onPause = () => setStatus("paused");
    const onWaiting = () => {
      setStatus((previous) =>
        previous === "playing" || previous === "loading" ? "loading" : previous,
      );
    };
    const onError = () => {
      setStatus("error");
      setPlaybackError("Could not play this track. The audio source failed to load.");
    };

    audio.addEventListener("timeupdate", onTimeUpdate);
    audio.addEventListener("durationchange", onDurationChange);
    audio.addEventListener("ended", onEnded);
    audio.addEventListener("playing", onPlaying);
    audio.addEventListener("pause", onPause);
    audio.addEventListener("waiting", onWaiting);
    audio.addEventListener("stalled", onWaiting);
    audio.addEventListener("error", onError);

    return () => {
      audio.removeEventListener("timeupdate", onTimeUpdate);
      audio.removeEventListener("durationchange", onDurationChange);
      audio.removeEventListener("ended", onEnded);
      audio.removeEventListener("playing", onPlaying);
      audio.removeEventListener("pause", onPause);
      audio.removeEventListener("waiting", onWaiting);
      audio.removeEventListener("stalled", onWaiting);
      audio.removeEventListener("error", onError);
      currentTrackIdRef.current = null;
      cleanup();
    };
  }, []);

  const play = useCallback((track: TrackDto) => {
    const audio = audioRef.current;
    if (!audio) return;
    if (currentTrackIdRef.current === track.id) {
      if (statusRef.current === "loading") return;
      if (audio.paused) {
        setPlaybackError(null);
        setStatus("loading");
        void Promise.resolve(audio.play()).catch((err: unknown) => {
          setStatus("error");
          setPlaybackError(formatPlaybackError("The audio engine rejected playback", err));
        });
      }
      return;
    }
    const requestId = playRequestRef.current + 1;
    playRequestRef.current = requestId;
    currentTrackIdRef.current = track.id;
    setCurrent(track);
    setPositionSeconds(0);
    setDurationSeconds(track.durationSeconds || 0);
    setPlaybackError(null);
    setStatus("loading");
    void loadAndPlayTrack(
      audio,
      track,
      requestId,
      playRequestRef,
      currentTrackIdRef,
      setStatus,
      setPlaybackError,
    );
  }, []);

  const toggle = useCallback(() => {
    const audio = audioRef.current;
    if (!audio || !current) return;
    if (audio.paused) {
      setPlaybackError(null);
      setStatus("loading");
      void Promise.resolve(audio.play()).catch((err: unknown) => {
        setStatus("error");
        setPlaybackError(formatPlaybackError("The audio engine rejected playback", err));
      });
    } else {
      audio.pause();
    }
  }, [current]);

  const seek = useCallback((seconds: number) => {
    const audio = audioRef.current;
    if (!audio) return;
    audio.currentTime = seconds;
    setPositionSeconds(seconds);
  }, []);

  const setVolume = useCallback((value: number) => {
    const audio = audioRef.current;
    if (!audio) return;
    const clamped = Math.min(1, Math.max(0, value));
    audio.volume = clamped;
    setVolumeState(clamped);
  }, []);

  return useMemo(
    () => ({
      current,
      status,
      positionSeconds,
      durationSeconds,
      volume,
      playbackError,
      play,
      toggle,
      seek,
      setVolume,
    }),
    [
      current,
      status,
      positionSeconds,
      durationSeconds,
      volume,
      playbackError,
      play,
      toggle,
      seek,
      setVolume,
    ],
  );
}

async function loadAndPlayTrack(
  audio: AudioElementLike,
  track: TrackDto,
  requestId: number,
  playRequestRef: React.MutableRefObject<number>,
  currentTrackIdRef: React.MutableRefObject<string | null>,
  setStatus: React.Dispatch<React.SetStateAction<PlayerStatus>>,
  setPlaybackError: React.Dispatch<React.SetStateAction<string | null>>,
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
  }
}

function createAudioElement(): { audio: AudioElementLike; cleanup: () => void } {
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

function formatPlaybackError(reason: string, err: unknown): string {
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
