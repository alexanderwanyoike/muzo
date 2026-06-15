import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { TrackDto } from "./api";

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
  const [current, setCurrent] = useState<TrackDto | null>(null);
  const [status, setStatus] = useState<PlayerStatus>("idle");
  const [positionSeconds, setPositionSeconds] = useState(0);
  const [durationSeconds, setDurationSeconds] = useState(0);
  const [volume, setVolumeState] = useState(1);
  const [playbackError, setPlaybackError] = useState<string | null>(null);

  useEffect(() => {
    const audio = createAudioElement();
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
    };
  }, []);

  const play = useCallback((track: TrackDto) => {
    const audio = audioRef.current;
    if (!audio) return;
    audio.src = convertFileSrc(track.filePath);
    audio.load();
    setCurrent(track);
    setPositionSeconds(0);
    setDurationSeconds(track.durationSeconds || 0);
    setPlaybackError(null);
    setStatus("loading");
    void Promise.resolve(audio.play()).catch(() => {
      setStatus("error");
      setPlaybackError("Could not play this track. The audio engine rejected playback.");
    });
  }, []);

  const toggle = useCallback(() => {
    const audio = audioRef.current;
    if (!audio || !current) return;
    if (audio.paused) {
      setPlaybackError(null);
      setStatus("loading");
      void Promise.resolve(audio.play()).catch(() => {
        setStatus("error");
        setPlaybackError("Could not play this track. The audio engine rejected playback.");
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

function createAudioElement(): AudioElementLike {
  if (typeof Audio !== "undefined") {
    return new Audio() as unknown as AudioElementLike;
  }
  return createStubAudioElement();
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
