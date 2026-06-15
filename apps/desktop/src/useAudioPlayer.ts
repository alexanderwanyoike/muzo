import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { TrackDto } from "./api";

export type PlayerStatus = "idle" | "playing" | "paused";

export interface AudioPlayerState {
  current: TrackDto | null;
  status: PlayerStatus;
  positionSeconds: number;
  durationSeconds: number;
  volume: number;
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

  useEffect(() => {
    const audio = createAudioElement();
    audioRef.current = audio;

    const onTimeUpdate = () => setPositionSeconds(audio.currentTime);
    const onDurationChange = () => setDurationSeconds(audio.duration || 0);
    const onEnded = () => setStatus("paused");
    const onPlay = () => setStatus("playing");
    const onPause = () => setStatus("paused");

    audio.addEventListener("timeupdate", onTimeUpdate);
    audio.addEventListener("durationchange", onDurationChange);
    audio.addEventListener("ended", onEnded);
    audio.addEventListener("play", onPlay);
    audio.addEventListener("pause", onPause);

    return () => {
      audio.removeEventListener("timeupdate", onTimeUpdate);
      audio.removeEventListener("durationchange", onDurationChange);
      audio.removeEventListener("ended", onEnded);
      audio.removeEventListener("play", onPlay);
      audio.removeEventListener("pause", onPause);
    };
  }, []);

  const play = useCallback((track: TrackDto) => {
    const audio = audioRef.current;
    if (!audio) return;
    audio.src = convertFileSrc(track.filePath);
    void audio.play();
    setCurrent(track);
    setPositionSeconds(0);
    setDurationSeconds(track.durationSeconds || 0);
  }, []);

  const toggle = useCallback(() => {
    const audio = audioRef.current;
    if (!audio || !current) return;
    if (audio.paused) {
      void audio.play();
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
    play: () => {
      state.paused = false;
      listeners.get("play")?.forEach((fn) => fn({}));
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
