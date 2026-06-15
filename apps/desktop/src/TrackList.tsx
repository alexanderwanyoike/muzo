import { useEffect, useState } from "react";
import { listTracks, type TrackDto } from "./api";
import { formatDuration } from "./formatDuration";

interface TrackListProps {
  libraryId: string;
  currentTrackId: string | null;
  isPlaying: boolean;
  onPlayTrack: (track: TrackDto) => void;
  refreshKey?: number;
  onTracksLoaded?: (libraryId: string, trackCount: number) => void;
}

export default function TrackList({
  libraryId,
  currentTrackId,
  isPlaying,
  onPlayTrack,
  refreshKey = 0,
  onTracksLoaded,
}: TrackListProps) {
  const [tracks, setTracks] = useState<TrackDto[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setError(null);
    listTracks(libraryId)
      .then((result) => {
        if (!cancelled) {
          setTracks(result);
          onTracksLoaded?.(libraryId, result.length);
        }
      })
      .catch((err: { message?: string }) => {
        if (!cancelled) {
          setError(err.message ?? "Could not load tracks.");
          setTracks([]);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [libraryId, onTracksLoaded, refreshKey]);

  if (error) {
    return <p className="track-list__error">{error}</p>;
  }
  if (tracks === null) {
    return <p className="track-list__placeholder">Loading tracks...</p>;
  }
  if (tracks.length === 0) {
    return (
      <p className="track-list__empty">
        No tracks yet. Hit Scan on the library to populate it.
      </p>
    );
  }

  return (
    <ol className="track-list">
      {tracks.map((track, index) => {
        const isCurrent = track.id === currentTrackId;
        const rowClass = isCurrent
          ? "track-list__row track-list__row--current"
          : "track-list__row";
        return (
          <li
            key={track.id}
            className={rowClass}
            data-track-id={track.id}
          >
            <button
              type="button"
              className="track-list__play"
              onClick={() => onPlayTrack(track)}
              aria-label={`Play ${track.title}`}
            >
              {isCurrent && isPlaying ? "❚❚" : "▶"}
            </button>
            <span className="track-list__index">{index + 1}</span>
            <div className="track-list__meta">
              <div className="track-list__title">{track.title}</div>
              <div className="track-list__artist">{track.artist}</div>
            </div>
            <span className="track-list__duration">
              {formatDuration(track.durationSeconds)}
            </span>
          </li>
        );
      })}
    </ol>
  );
}
