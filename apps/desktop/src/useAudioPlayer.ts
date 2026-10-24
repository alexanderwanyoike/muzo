import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { TrackDto } from "./api";
import { createAudioElement, formatPlaybackError, loadAndPlayTrack, type AudioElementLike } from "./audio-player-engine";

export type PlayerStatus = "idle" | "loading" | "playing" | "paused" | "error";

export interface AudioPlayerState {
  current: TrackDto | null;
  status: PlayerStatus;
  positionSeconds: number;
  durationSeconds: number;
  volume: number;
  playbackError: string | null;
}

export interface PlaybackQueueItem {
  entryId: string;
  track: TrackDto;
}

export interface AudioPlayer extends AudioPlayerState {
  playQueue: (entries: PlaybackQueueItem[], startIndex?: number) => void;
  currentQueueEntryId: string | null;
  hasNext: boolean;
  hasPrevious: boolean;
  next: () => void;
  previous: () => void;
  play: (track: TrackDto) => void;
  toggle: () => void;
  clearQueue: () => void;
  seek: (seconds: number) => void;
  setVolume: (volume: number) => void;
}

interface UseAudioPlayerOptions {
  onTrackPlayRecorded?: (track: TrackDto) => void;
}


export function useAudioPlayer(options: UseAudioPlayerOptions = {}): AudioPlayer {
  const { onTrackPlayRecorded } = options;
  const queueRef = useRef<{ entries: PlaybackQueueItem[]; index: number }>({ entries: [], index: -1 });
  const [queue, setQueueState] = useState(queueRef.current);
  const advanceRef = useRef<() => boolean>(() => false);
  const setQueue = useCallback((entries: PlaybackQueueItem[], index: number) => {
    const state = { entries, index };
    queueRef.current = state;
    setQueueState(state);
  }, []);
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
    const onEnded = () => {
      if (!advanceRef.current()) setStatus("paused");
    };
    const onPlaying = () => {
      setPlaybackError(null);
      setStatus("playing");
    };
    const onPause = () => {
      if (audio.paused && statusRef.current !== "loading") setStatus("paused");
    };
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
      playRequestRef.current += 1;
      audio.pause();
      audio.src = "";
      audio.load();
      cleanup();
    };
  }, []);

  const startTrack = useCallback((track: TrackDto) => {
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
    statusRef.current = "loading";
    audio.pause();
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
      onTrackPlayRecorded,
    );
  }, [onTrackPlayRecorded]);

  const clearQueue = useCallback(() => { setQueue([], -1); }, [setQueue]);

  const play = useCallback((track: TrackDto) => {
    clearQueue();
    startTrack(track);
  }, [clearQueue, startTrack]);

  const playQueue = useCallback((entries: PlaybackQueueItem[], startIndex = 0) => {
    if (!Number.isInteger(startIndex) || !entries[startIndex]) return;
    const snapshot = [...entries];
    setQueue(snapshot, startIndex);
    currentTrackIdRef.current = null;
    startTrack(snapshot[startIndex].track);
  }, [setQueue, startTrack]);


  const moveQueue = useCallback((direction: number): boolean => {
    const state = queueRef.current;
    const index = state.index + direction;
    const entry = state.entries[index];
    if (!entry) return false;
    setQueue(state.entries, index);
    currentTrackIdRef.current = null;
    startTrack(entry.track);
    return true;
  }, [setQueue, startTrack]);

  useEffect(() => {
    advanceRef.current = () => moveQueue(1);
  }, [moveQueue]);

  const next = useCallback(() => { moveQueue(1); }, [moveQueue]);
  const previous = useCallback(() => { moveQueue(-1); }, [moveQueue]);

  const toggle = useCallback(() => {
    const audio = audioRef.current;
    if (!audio || !current || statusRef.current === "loading") return;
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
      currentQueueEntryId: queue.entries[queue.index]?.entryId ?? null,
      hasNext: queue.index + 1 < queue.entries.length,
      hasPrevious: queue.index > 0,
      next,
      previous,
      playQueue,
      clearQueue,
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
      queue,
      next,
      previous,
      playQueue,
      clearQueue,
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
