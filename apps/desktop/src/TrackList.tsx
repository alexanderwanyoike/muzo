import { type FormEvent, useEffect, useState } from "react";
import {
  clearTrackMetadataOverride,
  editTrackMetadata,
  listTracks,
  type TrackDto,
} from "./api";
import { formatDuration } from "./formatDuration";

interface TrackListProps {
  libraryId: string;
  currentTrackId: string | null;
  isPlaying: boolean;
  onPlayTrack: (track: TrackDto) => void;
  onToggleCurrentTrack: () => void;
  refreshKey?: number;
  onTracksLoaded?: (libraryId: string, trackCount: number) => void;
}

export default function TrackList({
  libraryId,
  currentTrackId,
  isPlaying,
  onPlayTrack,
  onToggleCurrentTrack,
  refreshKey = 0,
  onTracksLoaded,
}: TrackListProps) {
  const [tracks, setTracks] = useState<TrackDto[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [editingTrack, setEditingTrack] = useState<TrackDto | null>(null);
  const [draft, setDraft] = useState<MetadataDraft | null>(null);
  const [saveError, setSaveError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setError(null);
    setEditingTrack(null);
    setDraft(null);
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

  async function reloadTracks() {
    const result = await listTracks(libraryId);
    setTracks(result);
    onTracksLoaded?.(libraryId, result.length);
    return result;
  }

  function startEditing(track: TrackDto) {
    setSaveError(null);
    setEditingTrack(track);
    setDraft({
      title: track.title,
      artist: track.artist,
      album: track.album ?? "",
      trackNumber: numberToDraft(track.trackNumber),
      discNumber: numberToDraft(track.discNumber),
      genre: track.genre ?? "",
      year: numberToDraft(track.year),
    });
  }

  async function handleSave(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!editingTrack || !draft) {
      return;
    }

    setSaveError(null);
    try {
      await editTrackMetadata({
        libraryId,
        trackId: editingTrack.id,
        title: textToMetadata(draft.title),
        artist: textToMetadata(draft.artist),
        album: textToMetadata(draft.album),
        trackNumber: numberFromDraft(draft.trackNumber),
        discNumber: numberFromDraft(draft.discNumber),
        genre: textToMetadata(draft.genre),
        year: numberFromDraft(draft.year),
      });
      await reloadTracks();
      setEditingTrack(null);
      setDraft(null);
    } catch (err) {
      const e = err as { message?: string };
      setSaveError(e.message ?? "Could not save metadata.");
    }
  }

  async function handleClearOverride() {
    if (!editingTrack) {
      return;
    }

    setSaveError(null);
    try {
      await clearTrackMetadataOverride({
        libraryId,
        trackId: editingTrack.id,
      });
      await reloadTracks();
      setEditingTrack(null);
      setDraft(null);
    } catch (err) {
      const e = err as { message?: string };
      setSaveError(e.message ?? "Could not clear metadata override.");
    }
  }

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
        const isEditing = editingTrack?.id === track.id && draft;
        const playButtonLabel =
          isCurrent && isPlaying ? `Pause ${track.title}` : `Play ${track.title}`;
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
              onClick={() => {
                if (isCurrent) {
                  onToggleCurrentTrack();
                } else {
                  onPlayTrack(track);
                }
              }}
              aria-label={playButtonLabel}
            >
              {isCurrent && isPlaying ? "❚❚" : "▶"}
            </button>
            <span className="track-list__index">{index + 1}</span>
            <div className="track-list__meta">
              <div className="track-list__title">{track.title}</div>
              <div className="track-list__artist">{trackSubtitle(track)}</div>
            </div>
            <span className="track-list__duration">
              {formatDuration(track.durationSeconds)}
            </span>
            <button
              type="button"
              className="track-list__edit"
              onClick={() => startEditing(track)}
              aria-label={`Edit ${track.title}`}
            >
              Edit
            </button>
            {isEditing && (
              <form className="track-list__editor" onSubmit={handleSave}>
                <label>
                  Title
                  <input
                    value={draft.title}
                    onChange={(event) =>
                      setDraft({ ...draft, title: event.target.value })
                    }
                  />
                </label>
                <label>
                  Artist
                  <input
                    value={draft.artist}
                    onChange={(event) =>
                      setDraft({ ...draft, artist: event.target.value })
                    }
                  />
                </label>
                <label>
                  Album
                  <input
                    value={draft.album}
                    onChange={(event) =>
                      setDraft({ ...draft, album: event.target.value })
                    }
                  />
                </label>
                <label>
                  Track
                  <input
                    min="1"
                    type="number"
                    value={draft.trackNumber}
                    onChange={(event) =>
                      setDraft({ ...draft, trackNumber: event.target.value })
                    }
                  />
                </label>
                <label>
                  Disc
                  <input
                    min="1"
                    type="number"
                    value={draft.discNumber}
                    onChange={(event) =>
                      setDraft({ ...draft, discNumber: event.target.value })
                    }
                  />
                </label>
                <label>
                  Genre
                  <input
                    value={draft.genre}
                    onChange={(event) =>
                      setDraft({ ...draft, genre: event.target.value })
                    }
                  />
                </label>
                <label>
                  Year
                  <input
                    type="number"
                    value={draft.year}
                    onChange={(event) =>
                      setDraft({ ...draft, year: event.target.value })
                    }
                  />
                </label>
                {saveError && (
                  <p role="alert" className="track-list__save-error">
                    {saveError}
                  </p>
                )}
                <div className="track-list__editor-actions">
                  <button type="submit">Save metadata</button>
                  <button type="button" onClick={handleClearOverride}>
                    Clear override
                  </button>
                  <button
                    type="button"
                    onClick={() => {
                      setEditingTrack(null);
                      setDraft(null);
                      setSaveError(null);
                    }}
                  >
                    Cancel
                  </button>
                </div>
              </form>
            )}
          </li>
        );
      })}
    </ol>
  );
}

interface MetadataDraft {
  title: string;
  artist: string;
  album: string;
  trackNumber: string;
  discNumber: string;
  genre: string;
  year: string;
}

function trackSubtitle(track: TrackDto): string {
  const source = track.album ?? (track.metadataOverridden ? "Edited in Muzo" : null);
  return source ? `${track.artist} - ${source}` : track.artist;
}

function textToMetadata(value: string): string | null {
  const trimmed = value.trim();
  return trimmed.length === 0 ? null : trimmed;
}

function numberToDraft(value: number | null): string {
  return value === null ? "" : String(value);
}

function numberFromDraft(value: string): number | null {
  if (value.trim().length === 0) {
    return null;
  }
  return Number(value);
}
