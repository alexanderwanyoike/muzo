import { type FormEvent, useEffect, useState } from "react";
import {
  createPlaylist,
  listPlaylists,
  removePlaylistEntry,
  reorderPlaylistEntries,
  type PlaylistDto,
  type PlaylistEntryDto,
  type TrackDto,
} from "./api";

interface PlaylistError {
  message?: string;
}

interface PlaylistPanelProps {
  currentTrackId?: string | null;
  isPlaying?: boolean;
  onPlayTrack?: (track: TrackDto) => void;
  onToggleCurrentTrack?: () => void;
}

export default function PlaylistPanel({
  currentTrackId = null,
  isPlaying = false,
  onPlayTrack = () => {},
  onToggleCurrentTrack = () => {},
}: PlaylistPanelProps = {}) {
  const [playlists, setPlaylists] = useState<PlaylistDto[] | null>(null);
  const [name, setName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  const [removingEntryId, setRemovingEntryId] = useState<string | null>(null);
  const [reorderingEntryId, setReorderingEntryId] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setError(null);
    listPlaylists()
      .then((result) => {
        if (!cancelled) {
          setPlaylists(result);
        }
      })
      .catch((err: PlaylistError) => {
        if (!cancelled) {
          setError(err.message ?? "Could not load playlists.");
          setPlaylists([]);
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  async function handleCreate(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const trimmedName = name.trim();
    if (trimmedName.length === 0) {
      setError("Enter a playlist name.");
      return;
    }

    setCreating(true);
    setError(null);
    try {
      const playlist = await createPlaylist({ name: trimmedName });
      setPlaylists((prev) => [...(prev ?? []), playlist]);
      setName("");
    } catch (err) {
      const e = err as PlaylistError;
      setError(e.message ?? "Could not create playlist.");
    } finally {
      setCreating(false);
    }
  }

  async function handleRemoveEntry(playlist: PlaylistDto, entry: PlaylistEntryDto) {
    setRemovingEntryId(entry.id);
    setError(null);
    try {
      const updated = await removePlaylistEntry({
        playlistId: playlist.id,
        entryId: entry.id,
      });
      setPlaylists((prev) =>
        (prev ?? []).map((candidate) =>
          candidate.id === updated.id ? updated : candidate,
        ),
      );
    } catch (err) {
      const e = err as PlaylistError;
      setError(e.message ?? "Could not remove playlist entry.");
    } finally {
      setRemovingEntryId(null);
    }
  }

  async function handleMoveEntry(
    playlist: PlaylistDto,
    entry: PlaylistEntryDto,
    direction: "up" | "down",
  ) {
    const currentIndex = playlist.entries.findIndex(
      (candidate) => candidate.id === entry.id,
    );
    const targetIndex = direction === "up" ? currentIndex - 1 : currentIndex + 1;
    if (
      currentIndex < 0 ||
      targetIndex < 0 ||
      targetIndex >= playlist.entries.length
    ) {
      return;
    }

    const reorderedEntries = [...playlist.entries];
    const [movedEntry] = reorderedEntries.splice(currentIndex, 1);
    reorderedEntries.splice(targetIndex, 0, movedEntry);

    setReorderingEntryId(entry.id);
    setError(null);
    try {
      const updated = await reorderPlaylistEntries({
        playlistId: playlist.id,
        orderedEntryIds: reorderedEntries.map((candidate) => candidate.id),
      });
      setPlaylists((prev) =>
        (prev ?? []).map((candidate) =>
          candidate.id === updated.id ? updated : candidate,
        ),
      );
    } catch (err) {
      const e = err as PlaylistError;
      setError(e.message ?? "Could not reorder playlist entries.");
    } finally {
      setReorderingEntryId(null);
    }
  }

  function handlePlayEntry(entry: PlaylistEntryDto) {
    if (entry.trackId === currentTrackId) {
      onToggleCurrentTrack();
      return;
    }

    const track = trackFromEntry(entry);
    if (track) {
      onPlayTrack(track);
    }
  }

  return (
    <div className="playlist-panel">
      <section className="playlist-panel__create">
        <form className="playlist-panel__form" onSubmit={handleCreate}>
          <label>
            Playlist name
            <input
              value={name}
              onChange={(event) => setName(event.target.value)}
              placeholder="New playlist"
            />
          </label>
          <button type="submit" disabled={creating}>
            {creating ? "Creating..." : "Create playlist"}
          </button>
        </form>
        {error && (
          <p role="alert" className="playlist-panel__error">
            {error}
          </p>
        )}
      </section>

      {playlists === null ? (
        <p className="app__placeholder">Loading playlists...</p>
      ) : playlists.length === 0 ? (
        <div className="playlist-panel__empty">
          <h3>No playlists yet</h3>
          <p>Create one for albums, moods, or listening queues.</p>
        </div>
      ) : (
        <ol className="playlist-panel__list">
          {playlists.map((playlist) => (
            <li key={playlist.id}>
              <div className="playlist-panel__summary">
                <h3>{playlist.name}</h3>
                <p>{formatTrackCount(playlist.entries.length)}</p>
              </div>
              {playlist.entries.length === 0 ? (
                <p className="playlist-panel__empty-entries">
                  No tracks in this playlist.
                </p>
              ) : (
                <ol className="playlist-panel__entries">
                  {playlist.entries.map((entry, index) => (
                    <li key={entry.id}>
                      <div>
                        <span data-testid="playlist-entry-label">
                          {entryLabel(entry)}
                        </span>
                        {entry.trackArtist && <small>{entry.trackArtist}</small>}
                      </div>
                      <div className="playlist-panel__entry-actions">
                        <button
                          type="button"
                          className="playlist-panel__entry-play"
                          disabled={!trackFromEntry(entry)}
                          onClick={() => handlePlayEntry(entry)}
                          aria-label={playButtonLabel(
                            entry,
                            playlist,
                            currentTrackId,
                            isPlaying,
                          )}
                        >
                          {entry.trackId === currentTrackId && isPlaying
                            ? "||"
                            : ">"}
                        </button>
                        <button
                          type="button"
                          disabled={
                            index === 0 || reorderingEntryId === entry.id
                          }
                          onClick={() => handleMoveEntry(playlist, entry, "up")}
                          aria-label={`Move ${entryLabel(entry)} up in ${playlist.name}`}
                        >
                          ^
                        </button>
                        <button
                          type="button"
                          disabled={
                            index === playlist.entries.length - 1 ||
                            reorderingEntryId === entry.id
                          }
                          onClick={() => handleMoveEntry(playlist, entry, "down")}
                          aria-label={`Move ${entryLabel(entry)} down in ${playlist.name}`}
                        >
                          v
                        </button>
                        <button
                          type="button"
                          disabled={removingEntryId === entry.id}
                          onClick={() => handleRemoveEntry(playlist, entry)}
                          aria-label={`Remove ${entryLabel(entry)} from ${playlist.name}`}
                        >
                          {removingEntryId === entry.id ? "Removing..." : "Remove"}
                        </button>
                      </div>
                    </li>
                  ))}
                </ol>
              )}
            </li>
          ))}
        </ol>
      )}
    </div>
  );
}

function formatTrackCount(count: number): string {
  return `${count} ${count === 1 ? "track" : "tracks"}`;
}

function entryLabel(entry: PlaylistEntryDto): string {
  return entry.trackTitle ?? entry.trackId;
}

function playButtonLabel(
  entry: PlaylistEntryDto,
  playlist: PlaylistDto,
  currentTrackId: string | null,
  isPlaying: boolean,
): string {
  const label = entryLabel(entry);
  if (!trackFromEntry(entry)) {
    return `Track ${label} is unavailable in ${playlist.name}`;
  }
  if (entry.trackId === currentTrackId && isPlaying) {
    return `Pause ${label} from ${playlist.name}`;
  }
  return `Play ${label} from ${playlist.name}`;
}

function trackFromEntry(entry: PlaylistEntryDto): TrackDto | null {
  if (!entry.trackLibraryId) {
    return null;
  }

  return {
    id: entry.trackId,
    libraryId: entry.trackLibraryId,
    title: entryLabel(entry),
    artist: entry.trackArtist ?? "Unknown Artist",
    album: null,
    trackNumber: null,
    discNumber: null,
    genre: null,
    year: null,
    metadataOverridden: false,
    durationSeconds: entry.trackDurationSeconds ?? 0,
    filePath: "",
  };
}
