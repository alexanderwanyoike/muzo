import playlistRequest from "./playlist-request";
import { createPortal } from "react-dom";
import PlaylistNavigator from "./playlist-navigator";
import { Play, Pause, ChevronUp, ChevronDown } from "lucide-react";
import { useEffect, useState } from "react";
import TrackArtwork from "./track-artwork";
import {
  createPlaylist,
  listPlaylists,
  removePlaylistEntry,
  reorderPlaylistEntries,
  type PlaylistDto,
  type PlaylistEntryDto,
  type TrackDto,
} from "./api";

import type { PlaybackQueueItem } from "./useAudioPlayer";
import {
  entryLabel,
  isCurrentEntry,
  playableEntries,
  playButtonLabel,
  trackFromEntry,
} from "./playlist-playback";

interface PlaylistError {
  message?: string;
}

interface PlaylistPanelProps {
  navigationTarget?: HTMLElement | null;
  refreshKey?: number;
  currentQueueEntryId?: string | null;
  currentTrackId?: string | null;
  isPlaying?: boolean;
  onPlayQueue?: (entries: PlaybackQueueItem[], startIndex: number) => void;
  onPlayTrack?: (track: TrackDto) => void;
  onToggleCurrentTrack?: () => void;
}

export default function PlaylistPanel({
  navigationTarget,
  refreshKey = 0,
  currentTrackId = null,
  currentQueueEntryId = null,
  isPlaying = false,
  onPlayTrack = () => {},
  onPlayQueue,
  onToggleCurrentTrack = () => {},
}: PlaylistPanelProps = {}) {
  const activeEntryId = onPlayQueue ? currentQueueEntryId : undefined;
  const [playlists, setPlaylists] = useState<PlaylistDto[] | null>(null);
  const [selectedId, selectPlaylist] = useState<string | null>(null);
  const selected = playlists?.find((playlist) => playlist.id === selectedId) ?? playlists?.[0];
  const [error, setError] = useState<string | null>(null);
  const [removingEntryId, setRemovingEntryId] = useState<string | null>(null);
  const [reorderingEntryId, setReorderingEntryId] = useState<string | null>(
    null,
  );

  useEffect(() => {
    let cancelled = false;
    setPlaylists(null);
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
  }, [refreshKey]);

  async function handleCreate(name: string) {
    const result = await playlistRequest(() => createPlaylist({ name }), "Could not create playlist.");
    if (!result.ok) return result;
    const playlist = result.value;
    setPlaylists((prev) => [...(prev ?? []), playlist]);
    selectPlaylist(playlist.id);
    return { ok: true as const, value: undefined };
  }

  async function handleRemoveEntry(
    playlist: PlaylistDto,
    entry: PlaylistEntryDto,
  ) {
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
    const targetIndex =
      direction === "up" ? currentIndex - 1 : currentIndex + 1;
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

  function handlePlayEntry(playlist: PlaylistDto, entry: PlaylistEntryDto) {
    if (isCurrentEntry(entry, currentTrackId, activeEntryId)) {
      onToggleCurrentTrack();
      return;
    }

    const track = trackFromEntry(entry);
    if (track) {
      if (onPlayQueue) {
        const entries = playableEntries(playlist);
        onPlayQueue(
          entries,
          entries.findIndex((candidate) => candidate.entryId === entry.id),
        );
      } else onPlayTrack(track);
    }
  }

  function handlePlayPlaylist(playlist: PlaylistDto) {
    const entries = playableEntries(playlist);
    if (!entries.length) return;
    if (onPlayQueue) onPlayQueue(entries, 0);
    else onPlayTrack(entries[0].track);
  }

  const navigator = <PlaylistNavigator playlists={playlists} selectedId={selected?.id ?? null} onSelect={selectPlaylist} onCreate={handleCreate} />;
  return (
    <div className="playlist-panel">
      {navigationTarget ? createPortal(navigator, navigationTarget) : navigator}
      {error && <p role="alert" className="playlist-panel__error">{error}</p>}

      {playlists === null ? (
        <p className="app__placeholder">Loading playlists...</p>
      ) : playlists.length === 0 ? (
        <div className="playlist-panel__empty">
          <h3>No playlists yet</h3>
          <p>Create one for albums, moods, or listening queues.</p>
        </div>
      ) : (
        <ol className="playlist-panel__list">
          {playlists.filter((playlist) => playlist.id === selected?.id).map((playlist) => (
            <li key={playlist.id}>
              <div className="playlist-panel__summary">
                <h3>{playlist.name}</h3>
                <p>{formatTrackCount(playlist.entries.length)}</p>
                <button
                  type="button"
                  disabled={!playableEntries(playlist).length}
                  aria-label={`Play ${playlist.name}`}
                  onClick={() => handlePlayPlaylist(playlist)}
                >
                  Play playlist
                </button>
              </div>
              {playlist.entries.length === 0 ? (
                <p className="playlist-panel__empty-entries">
                  No tracks in this playlist.
                </p>
              ) : (
                <ol className="playlist-panel__entries">
                  {playlist.entries.map((entry, index) => (
                    <li key={entry.id}>
                      {entry.trackLibraryId && (
                        <TrackArtwork
                          track={{
                            id: entry.trackId,
                            libraryId: entry.trackLibraryId,
                            title: entryLabel(entry),
                            album: null,
                          }}
                        />
                      )}
                      <div className="playlist-panel__entry-meta">
                        <span data-testid="playlist-entry-label">
                          {entryLabel(entry)}
                        </span>
                        {!entry.trackLibraryId && (
                          <small>Unavailable source</small>
                        )}
                        {entry.trackArtist && (
                          <small>{entry.trackArtist}</small>
                        )}
                      </div>
                      <div className="playlist-panel__entry-actions">
                        <button
                          type="button"
                          className="playlist-panel__entry-play"
                          disabled={!trackFromEntry(entry)}
                          onClick={() => handlePlayEntry(playlist, entry)}
                          aria-label={playButtonLabel(
                            entry,
                            playlist,
                            currentTrackId,
                            isPlaying,
                            activeEntryId,
                          )}
                        >
                          {isCurrentEntry(
                            entry,
                            currentTrackId,
                            activeEntryId,
                          ) && isPlaying ? (
                            <Pause aria-hidden="true" />
                          ) : (
                            <Play aria-hidden="true" />
                          )}
                        </button>
                        <button
                          type="button"
                          disabled={
                            index === 0 || reorderingEntryId === entry.id
                          }
                          onClick={() => handleMoveEntry(playlist, entry, "up")}
                          aria-label={`Move ${entryLabel(entry)} up in ${playlist.name}`}
                        >
                          <ChevronUp aria-hidden="true" />
                        </button>
                        <button
                          type="button"
                          disabled={
                            index === playlist.entries.length - 1 ||
                            reorderingEntryId === entry.id
                          }
                          onClick={() =>
                            handleMoveEntry(playlist, entry, "down")
                          }
                          aria-label={`Move ${entryLabel(entry)} down in ${playlist.name}`}
                        >
                          <ChevronDown aria-hidden="true" />
                        </button>
                        <button
                          type="button"
                          disabled={removingEntryId === entry.id}
                          onClick={() => handleRemoveEntry(playlist, entry)}
                          aria-label={`Remove ${entryLabel(entry)} from ${playlist.name}`}
                        >
                          {removingEntryId === entry.id
                            ? "Removing..."
                            : "Remove"}
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
