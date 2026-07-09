import { type FormEvent, useEffect, useState } from "react";
import {
  createPlaylist,
  listPlaylists,
  removePlaylistEntry,
  type PlaylistDto,
  type PlaylistEntryDto,
} from "./api";

interface PlaylistError {
  message?: string;
}

export default function PlaylistPanel() {
  const [playlists, setPlaylists] = useState<PlaylistDto[] | null>(null);
  const [name, setName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  const [removingEntryId, setRemovingEntryId] = useState<string | null>(null);

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
                  {playlist.entries.map((entry) => (
                    <li key={entry.id}>
                      <div>
                        <span>{entryLabel(entry)}</span>
                        {entry.trackArtist && <small>{entry.trackArtist}</small>}
                      </div>
                      <button
                        type="button"
                        disabled={removingEntryId === entry.id}
                        onClick={() => handleRemoveEntry(playlist, entry)}
                        aria-label={`Remove ${entryLabel(entry)} from ${playlist.name}`}
                      >
                        {removingEntryId === entry.id ? "Removing..." : "Remove"}
                      </button>
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
