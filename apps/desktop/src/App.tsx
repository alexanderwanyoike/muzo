import { Music2, ListMusic, Settings2, Minimize2 } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import CompactPlayer from "./compact-player";
import { usePlayerWindowMode } from "./use-player-window-mode";
import SettingsSection from "./settings-section";
import SidebarSplitter from "./sidebar-splitter";
import { useSidebarLayout } from "./use-sidebar-layout";
import TrackArtwork from "./track-artwork";
import RemoveSourceDialog from "./remove-source-dialog";
import AppearanceSettings from "./appearance-settings";
import { useAppearance } from "./use-appearance";
import DropboxSettings from "./dropbox-settings";
import AddLibraryForm from "./AddLibraryForm";
import SourcesPanel from "./sources-panel";
import TrackList from "./TrackList";
import PlaylistPanel from "./playlist-panel";
import NowPlayingBar from "./NowPlayingBar";
import { useAudioPlayer } from "./useAudioPlayer";
import {
  listLibraries,
  listTrackPlayCounts,
  listTracks,
  scanLibrary,
} from "./api";
import { syncDropboxLibrary } from "./dropbox-api";
import type { LibraryDto } from "./types";

type ActiveView = "library" | "playlists" | "settings";

interface ListError {
  kind: string;
  message?: string;
}

export default function App() {
  const sidebar = useSidebarLayout();
  const playerWindow = usePlayerWindowMode();
  const miniPlayerButton = useRef<HTMLButtonElement>(null);
  const previousWindowMode = useRef(playerWindow.mode);
  useEffect(() => {
    if (
      previousWindowMode.current !== "library" &&
      playerWindow.mode === "library"
    )
      miniPlayerButton.current?.focus();
    previousWindowMode.current = playerWindow.mode;
  }, [playerWindow.mode]);
  const [playlistNavigationTarget, setPlaylistNavigationTarget] =
    useState<HTMLDivElement | null>(null);
  const sourceSetup = useRef<HTMLElement>(null);
  const [sourceSettingsRequest, requestSourceSettings] = useState(0);
  useEffect(() => {
    if (sourceSettingsRequest > 0) sourceSetup.current?.focus();
  }, [sourceSettingsRequest]);
  const removedLibraryIds = useRef(new Set<string>());
  const {
    appearance,
    changeAppearance,
    error: appearanceError,
  } = useAppearance();
  const [sourceRevision, setSourceRevision] = useState(0);
  const [removingLibrary, setRemovingLibrary] = useState<LibraryDto | null>(
    null,
  );
  const [libraries, setLibraries] = useState<LibraryDto[] | null>(null);
  const [trackCounts, setTrackCounts] = useState<Record<string, number>>({});
  const [trackPlayCounts, setTrackPlayCounts] = useState<
    Record<string, Record<string, number>>
  >({});
  const [trackRefreshVersions, setTrackRefreshVersions] = useState<
    Record<string, number>
  >({});
  const [refreshingLibraryId, setRefreshingLibraryId] = useState<string | null>(
    null,
  );
  const [refreshMessage, setRefreshMessage] = useState<string | null>(null);
  const [selectedLibraryId, setSelectedLibraryId] = useState<string | null>(
    null,
  );
  const [playlistsOpened, setPlaylistsOpened] = useState(false);
  const [activeView, setActiveView] = useState<ActiveView>("library");
  const [error, setError] = useState<string | null>(null);

  const handleTrackPlayRecorded = useCallback(
    (track: { libraryId: string; id: string }) => {
      setTrackPlayCounts((prev) => ({
        ...prev,
        [track.libraryId]: {
          ...(prev[track.libraryId] ?? {}),
          [track.id]: (prev[track.libraryId]?.[track.id] ?? 0) + 1,
        },
      }));
    },
    [],
  );

  const player = useAudioPlayer({
    onTrackPlayRecorded: handleTrackPlayRecorded,
  });
  const {
    current,
    status,
    positionSeconds,
    durationSeconds,
    volume,
    playbackError,
    play,
    playQueue,
    currentQueueEntryId,
    hasNext,
    hasPrevious,
    next,
    previous,
    toggle,
    clearQueue,
    removeSource,
    seek,
    setVolume,
  } = player;

  const refresh = async () => {
    setError(null);
    try {
      const libs = await listLibraries();
      setLibraries(libs);
    } catch (err) {
      const e = err as ListError;
      setError(e.message ?? "Could not load libraries.");
    }
  };

  useEffect(() => {
    refresh();
  }, []);

  useEffect(() => {
    if (!selectedLibraryId) {
      return;
    }

    let cancelled = false;
    listTrackPlayCounts(selectedLibraryId)
      .then((counts) => {
        if (cancelled) {
          return;
        }
        setTrackPlayCounts((prev) => ({
          ...prev,
          [selectedLibraryId]: Object.fromEntries(
            counts.map((count) => [count.trackId, count.playCount]),
          ),
        }));
      })
      .catch(() => {
        if (!cancelled) {
          setTrackPlayCounts((prev) => ({ ...prev, [selectedLibraryId]: {} }));
        }
      });

    return () => {
      cancelled = true;
    };
  }, [selectedLibraryId]);

  async function handleRefresh(library: LibraryDto) {
    setRefreshingLibraryId(library.id);
    setRefreshMessage(null);
    try {
      const remoteReport =
        library.kind === "dropbox"
          ? await syncDropboxLibrary(library.id)
          : null;
      if (removedLibraryIds.current.has(library.id)) return;
      if (remoteReport && !remoteReport.ok) {
        setRefreshMessage(remoteReport.error.message);
        return;
      }
      const report = remoteReport?.ok
        ? remoteReport.value
        : await scanLibrary(library.id);
      const tracks = await listTracks(library.id);
      if (removedLibraryIds.current.has(library.id)) return;
      setTrackCounts((prev) => ({ ...prev, [library.id]: tracks.length }));
      setTrackRefreshVersions((prev) => ({
        ...prev,
        [library.id]: (prev[library.id] ?? 0) + 1,
      }));
      setRefreshMessage(
        `Refreshed ${library.name}: ${report.tracksScanned} track${
          report.tracksScanned === 1 ? "" : "s"
        }.`,
      );
    } catch (err) {
      const e = err as ListError;
      if (!removedLibraryIds.current.has(library.id))
        setRefreshMessage(e.message ?? `Could not refresh ${library.name}.`);
    } finally {
      setRefreshingLibraryId(null);
    }
  }

  function handleSelect(libraryId: string | null) {
    setSelectedLibraryId(libraryId);
    setActiveView("library");
  }

  function rememberLibrary(library: LibraryDto) {
    setLibraries((items) => [...(items ?? []), library]);
    setTrackCounts((counts) => ({ ...counts, [library.id]: 0 }));
  }

  function handleLibraryAdded(library: LibraryDto) {
    rememberLibrary(library);
    setSelectedLibraryId(library.id);
    setActiveView("library");
  }

  const handleTracksLoaded = useCallback(
    (libraryId: string, trackCount: number) => {
      setTrackCounts((prev) => ({ ...prev, [libraryId]: trackCount }));
    },
    [],
  );

  function handleSourceRemoved(libraryId: string, warning: string | null) {
    removedLibraryIds.current.add(libraryId);
    setSourceRevision((revision) => revision + 1);
    removeSource(libraryId);
    setLibraries((items) =>
      (items ?? []).filter((library) => library.id !== libraryId),
    );
    setSelectedLibraryId((id) => (id === libraryId ? null : id));
    setTrackCounts((counts) =>
      Object.fromEntries(
        Object.entries(counts).filter(([id]) => id !== libraryId),
      ),
    );
    setRefreshMessage(warning);
  }

  const selectedLibrary =
    libraries?.find((library) => library.id === selectedLibraryId) ?? null;
  const totalTrackCount = Object.values(trackCounts).reduce(
    (sum, count) => sum + count,
    0,
  );

  return (
    <>
      {playerWindow.mode !== "library" && (
        <CompactPlayer
          player={player}
          queueOpen={playerWindow.mode === "compact-queue"}
          onToggleQueue={() => {
            void playerWindow.changeMode(
              playerWindow.mode === "compact-queue"
                ? "compact"
                : "compact-queue",
            );
          }}
          artworkRefreshKey={
            current ? (trackRefreshVersions[current.libraryId] ?? 0) : 0
          }
          pending={playerWindow.pending}
          error={playerWindow.error}
          onRestore={() => {
            void playerWindow.changeMode("library");
          }}
        />
      )}
      <main
        className="app-shell app--has-player"
        hidden={playerWindow.mode !== "library"}
        style={{ gridTemplateColumns: `${sidebar.width}px minmax(0, 1fr)` }}
      >
        <aside className="app-sidebar" id="source-sidebar">
          <header className="app-sidebar__header">
            <div>
              <h1>Muzo</h1>
              <p>
                {libraries?.length ?? 0} libraries - {totalTrackCount} tracks
              </p>
            </div>
            <button
              type="button"
              ref={miniPlayerButton}
              aria-label="Mini player"
              disabled={playerWindow.pending}
              onClick={() => {
                void playerWindow.changeMode("compact");
              }}
            >
              <Minimize2 aria-hidden="true" /> Mini player
            </button>
            {playerWindow.error && <p role="alert">{playerWindow.error}</p>}
          </header>

          <nav className="app-sidebar__nav" aria-label="Primary">
            <button
              type="button"
              className={
                activeView === "library"
                  ? "app-sidebar__nav-item app-sidebar__nav-item--active"
                  : "app-sidebar__nav-item"
              }
              onClick={() => setActiveView("library")}
            >
              <Music2 aria-hidden="true" /> Songs
            </button>
            <button
              type="button"
              className={
                activeView === "playlists"
                  ? "app-sidebar__nav-item app-sidebar__nav-item--active"
                  : "app-sidebar__nav-item"
              }
              onClick={() => {
                setPlaylistsOpened(true);
                setActiveView("playlists");
              }}
            >
              <ListMusic aria-hidden="true" /> Playlists
            </button>
            <button
              type="button"
              className={
                activeView === "settings"
                  ? "app-sidebar__nav-item app-sidebar__nav-item--active"
                  : "app-sidebar__nav-item"
              }
              onClick={() => setActiveView("settings")}
            >
              <Settings2 aria-hidden="true" /> Settings
            </button>
          </nav>

          <div
            className="app-sidebar__panel-slot"
            hidden={activeView === "playlists"}
          >
            <SourcesPanel
              libraries={libraries}
              trackCounts={trackCounts}
              selectedLibraryId={selectedLibraryId}
              onSelect={handleSelect}
              onRemove={setRemovingLibrary}
              onAddSource={() => {
                setActiveView("settings");
                requestSourceSettings((request) => request + 1);
              }}
            />
          </div>
          <div
            className="app-sidebar__panel-slot"
            hidden={activeView !== "playlists"}
            ref={setPlaylistNavigationTarget}
          />
          {current && (
            <figure
              className="app-sidebar__sleeve"
              aria-label="Current album artwork"
            >
              <TrackArtwork
                track={current}
                eager
                refreshKey={trackRefreshVersions[current.libraryId] ?? 0}
                className="track-artwork--large"
              />
            </figure>
          )}
          {sidebar.error && (
            <p className="app-sidebar__layout-error" aria-live="polite">
              {sidebar.error}
            </p>
          )}
          <SidebarSplitter
            panel={activeView === "playlists" ? "Playlists" : "Sources"}
            width={sidebar.width}
            minimum={sidebar.minimum}
            maximum={sidebar.maximum}
            onPreview={sidebar.preview}
            onCommit={sidebar.commit}
            onCancel={sidebar.cancel}
          />
        </aside>

        <section className="app-content">
          {activeView === "library" ? (
            <header className="app-content__toolbar">
              <div>
                <p className="app-content__eyebrow">Library</p>
                <h2>{selectedLibrary?.name ?? "Songs"}</h2>
              </div>
            </header>
          ) : activeView === "playlists" ? (
            <header className="app-content__toolbar">
              <div>
                <p className="app-content__eyebrow">Collections</p>
                <h2>Playlists</h2>
              </div>
            </header>
          ) : (
            <header className="app-content__toolbar">
              <div>
                <p className="app-content__eyebrow">Preferences</p>
                <h2>Settings</h2>
              </div>
            </header>
          )}

          {error && (
            <p role="alert" className="app__error">
              {error}
            </p>
          )}
          {refreshMessage && (
            <p role="status" className="app__info">
              {refreshMessage}
            </p>
          )}
          {playlistsOpened && (
            <div
              className="app-content__playlist-workspace"
              hidden={activeView !== "playlists"}
            >
              <PlaylistPanel
                active={activeView === "playlists"}
                navigationTarget={playlistNavigationTarget}
                refreshKey={sourceRevision}
                onPlayQueue={playQueue}
                currentQueueEntryId={currentQueueEntryId}
                currentTrackId={current?.id ?? null}
                isPlaying={status === "playing"}
                onPlayTrack={play}
                onToggleCurrentTrack={toggle}
              />
            </div>
          )}
          {activeView === "settings" ? (
            <div className="settings-view">
              <div className="settings-view__content">
                <AppearanceSettings
                  appearance={appearance}
                  onChange={changeAppearance}
                  error={appearanceError}
                />
                <section
                  className="settings-view__source-setup"
                  aria-label="Add sources"
                  tabIndex={-1}
                  ref={sourceSetup}
                >
                  <DropboxSettings
                    onLibraryAdded={(library) => {
                      rememberLibrary(library);
                      void handleRefresh(library);
                    }}
                  />
                  <SettingsSection
                    title="Library folders"
                    headingId="library-folders-heading"
                    description="Add folders here. Filesystem libraries sync automatically while the app is open and reconcile when it starts."
                  >
                    <AddLibraryForm onAdded={handleLibraryAdded} />
                  </SettingsSection>
                </section>
                <SettingsSection
                  title="Indexed sources"
                  headingId="indexed-sources-heading"
                  description="Manual refresh is a fallback for recovery. Normal listening does not require it."
                >
                  <ul className="settings-view__sources">
                    {(libraries ?? []).map((library) => (
                      <li key={library.id}>
                        <div className="settings-view__source-main">
                          <span>{library.name}</span>
                          <code>{library.location}</code>
                        </div>
                        <div className="settings-view__source-actions">
                          <button
                            type="button"
                            aria-label={`Refresh ${library.name}`}
                            disabled={refreshingLibraryId === library.id}
                            onClick={() => handleRefresh(library)}
                          >
                            {refreshingLibraryId === library.id
                              ? "Refreshing..."
                              : "Refresh"}
                          </button>
                          <button
                            type="button"
                            aria-label={`Remove ${library.name}`}
                            onClick={() => setRemovingLibrary(library)}
                          >
                            Remove
                          </button>
                        </div>
                      </li>
                    ))}
                  </ul>
                </SettingsSection>
              </div>
            </div>
          ) : activeView === "playlists" ? null : libraries === null ? (
            <p className="app__placeholder">Loading libraries...</p>
          ) : selectedLibrary ? (
            <div className="app-content__tracks">
              <TrackList
                libraryId={selectedLibrary.id}
                refreshIntervalMs={
                  selectedLibrary.kind === "dropbox" ? 5000 : undefined
                }
                currentTrackId={current?.id ?? null}
                isPlaying={status === "playing"}
                onPlayTrack={play}
                onToggleCurrentTrack={() => {
                  clearQueue();
                  toggle();
                }}
                refreshKey={trackRefreshVersions[selectedLibrary.id] ?? 0}
                onTracksLoaded={handleTracksLoaded}
                playCounts={trackPlayCounts[selectedLibrary.id] ?? {}}
              />
            </div>
          ) : (
            <div className="app-content__empty">
              <h2>No music yet</h2>
              <p>Add a folder in Settings. Muzo will sync it automatically.</p>
              <button
                type="button"
                className="app-content__empty-action"
                onClick={() => setActiveView("settings")}
              >
                Open Settings
              </button>
            </div>
          )}
        </section>

        {removingLibrary && (
          <RemoveSourceDialog
            library={removingLibrary}
            onRemoved={handleSourceRemoved}
            onClose={() => setRemovingLibrary(null)}
          />
        )}
        <NowPlayingBar
          artworkRefreshKey={
            current ? (trackRefreshVersions[current.libraryId] ?? 0) : 0
          }
          hasNext={hasNext}
          hasPrevious={hasPrevious}
          onNext={next}
          onPrevious={previous}
          current={current}
          status={status}
          positionSeconds={positionSeconds}
          durationSeconds={durationSeconds}
          volume={volume}
          playbackError={playbackError}
          onToggle={toggle}
          onSeek={seek}
          onSetVolume={setVolume}
        />
      </main>
    </>
  );
}
