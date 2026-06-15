import { formatDuration } from "./formatDuration";
import type { TrackDto } from "./api";

interface NowPlayingBarProps {
  current: TrackDto | null;
  status: "idle" | "loading" | "playing" | "paused" | "error";
  positionSeconds: number;
  durationSeconds: number;
  volume: number;
  playbackError: string | null;
  onToggle: () => void;
  onSeek: (seconds: number) => void;
  onSetVolume: (value: number) => void;
}

export default function NowPlayingBar({
  current,
  status,
  positionSeconds,
  durationSeconds,
  volume,
  playbackError,
  onToggle,
  onSeek,
  onSetVolume,
}: NowPlayingBarProps) {
  const progress = durationSeconds > 0 ? (positionSeconds / durationSeconds) * 100 : 0;
  const title = current?.title ?? "Not Playing";
  const artist = current?.artist ?? "Select a track";
  const isPlayable = current !== null;

  function handleProgressBarClick(event: React.MouseEvent<HTMLDivElement>) {
    if (!isPlayable || durationSeconds <= 0) return;
    const rect = event.currentTarget.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const fraction = x / rect.width;
    onSeek(fraction * durationSeconds);
  }

  return (
    <div className="now-playing-bar">
      <div className="now-playing-bar__track">
        <div className="now-playing-bar__title">{title}</div>
        <div className="now-playing-bar__artist">{artist}</div>
        {playbackError && (
          <div className="now-playing-bar__error">{playbackError}</div>
        )}
      </div>

      <div className="now-playing-bar__controls">
        <button
          type="button"
          className="now-playing-bar__play"
          onClick={onToggle}
          aria-label={status === "playing" ? "Pause" : "Play"}
          disabled={!isPlayable}
        >
          {status === "playing" ? "❚❚" : status === "loading" ? "…" : "▶"}
        </button>

        <span className="now-playing-bar__time">
          {formatDuration(positionSeconds)}
        </span>

        <div
          className="now-playing-bar__progress"
          onClick={handleProgressBarClick}
          role="slider"
          aria-label="Seek"
          aria-valuemin={0}
          aria-valuemax={Math.max(0, durationSeconds)}
          aria-valuenow={positionSeconds}
          aria-disabled={!isPlayable}
          tabIndex={0}
        >
          <div
            className="now-playing-bar__progress-fill"
            style={{ width: `${progress}%` }}
          />
        </div>

        <span className="now-playing-bar__time">
          {formatDuration(durationSeconds)}
        </span>
      </div>

      <div className="now-playing-bar__volume">
        <span className="now-playing-bar__volume-icon">♫</span>
        <input
          type="range"
          min={0}
          max={1}
          step={0.05}
          value={volume}
          onChange={(e) => onSetVolume(Number(e.target.value))}
          aria-label="Volume"
          className="now-playing-bar__volume-slider"
        />
      </div>
    </div>
  );
}
