import { useCallback, useEffect, useState } from "react";
import AddLibraryForm from "./AddLibraryForm";
import LibraryList from "./LibraryList";
import TrackList from "./TrackList";
import NowPlayingBar from "./NowPlayingBar";
import { useAudioPlayer } from "./useAudioPlayer";
import { listLibraries, listTrackPlayCounts, listTracks, scanLibrary } from "./api";
import type { LibraryDto } from "./types";

type ActiveView = "library" | "settings";

interface ListError {
  kind: string;
  message?: string;
}

export default function App() {
  const [libraries, setLibraries] = useState<LibraryDto[] | null>(null);
  const [trackCounts, setTrackCounts] = useState<Record<string, number>>({});
  const [trackPlayCounts, setTrackPlayCounts] = useState<Record<string, Record<string, number>>>({});
  const [trackRefreshVersions, setTrackRefreshVersions] = useState<Record<string, number>>({});
  const [scanningLibraryId, setScanningLibraryId] = useState<string | null>(null);
  const [scanMessage, setScanMessage] = useState<string | null>(null);
  const [selectedLibraryId, setSelectedLibraryId] = useState<string | null>(null);
  const [activeView, setActiveView] = useState<ActiveView>("library");
  const [error, setError] = useState<string | null>(null);

  const handleTrackPlayRecorded = useCallback((track: { libraryId: string; id: string }) => {
    setTrackPlayCounts((prev) => ({
      ...prev,
      [track.libraryId]: {
        ...(prev[track.libraryId] ?? {}),
        [track.id]: (prev[track.libraryId]?.[track.id] ?? 0) + 1,
      },
    }));
  }, []);

  const {
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
  } = useAudioPlayer({ onTrackPlayRecorded: handleTrackPlayRecorded });

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

  async function handleScan(libraryId: string) {
    setScanningLibraryId(libraryId);
    setScanMessage(null);
    try {
      const report = await scanLibrary(libraryId);
      const tracks = await listTracks(libraryId);
      setTrackCounts((prev) => ({ ...prev, [libraryId]: tracks.length }));
      setTrackRefreshVersions((prev) => ({
        ...prev,
        [libraryId]: (prev[libraryId] ?? 0) + 1,
      }));
      setScanMessage(
        `Scanned ${report.tracksScanned} track${
          report.tracksScanned === 1 ? "" : "s"
        }.`,
      );
    } catch (err) {
      const e = err as ListError;
      setScanMessage(e.message ?? "Scan failed.");
    } finally {
      setScanningLibraryId(null);
    }
  }

  function handleSelect(libraryId: string | null) {
    setSelectedLibraryId(libraryId);
    setActiveView("library");
  }

  function handleLibraryAdded(library: LibraryDto) {
    setLibraries((prev) => (prev ? [...prev, library] : [library]));
    setSelectedLibraryId(library.id);
    setActiveView("library");
    setTrackCounts((prev) => ({ ...prev, [library.id]: 0 }));
  }

  const handleTracksLoaded = useCallback((libraryId: string, trackCount: number) => {
    setTrackCounts((prev) => ({ ...prev, [libraryId]: trackCount }));
  }, []);

  const selectedLibrary =
    libraries?.find((library) => library.id === selectedLibraryId) ?? null;
  const totalTrackCount = Object.values(trackCounts).reduce(
    (sum, count) => sum + count,
    0,
  );

  return (
    <main className="app-shell app--has-player">
      <aside className="app-sidebar">
        <header className="app-sidebar__header">
          <div>
            <h1>Muzo</h1>
            <p>{libraries?.length ?? 0} libraries - {totalTrackCount} tracks</p>
          </div>
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
            Songs
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
            Settings
          </button>
        </nav>

        <section className="app-sidebar__section">
          <h2>Sources</h2>
          {libraries === null ? (
            <p className="app__placeholder">Loading...</p>
          ) : (
            <LibraryList
              libraries={libraries}
              trackCounts={trackCounts}
              scanningLibraryId={scanningLibraryId}
              selectedLibraryId={selectedLibraryId}
              onScan={handleScan}
              onSelect={handleSelect}
              renderTracks={() => null}
            />
          )}
        </section>
      </aside>

      <section className="app-content">
        {activeView === "library" ? (
          <header className="app-content__toolbar">
            <div>
              <p className="app-content__eyebrow">Library</p>
              <h2>{selectedLibrary?.name ?? "Songs"}</h2>
            </div>
            {selectedLibrary && (
              <button
                type="button"
                className="app-content__scan"
                disabled={scanningLibraryId === selectedLibrary.id}
                onClick={() => handleScan(selectedLibrary.id)}
                aria-label="Scan selected library"
              >
                {scanningLibraryId === selectedLibrary.id ? "Scanning..." : "Scan"}
              </button>
            )}
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
        {scanMessage && (
          <p role="status" className="app__info">
            {scanMessage}
          </p>
        )}
        {activeView === "settings" ? (
          <div className="settings-view">
            <section className="settings-view__section">
              <div>
                <h3>Library folders</h3>
                <p>
                  Add folders here. The listening view stays focused on browsing
                  and playback.
                </p>
              </div>
              <AddLibraryForm onAdded={handleLibraryAdded} />
            </section>

            <section className="settings-view__section">
              <div>
                <h3>Indexed sources</h3>
                <p>{libraries?.length ?? 0} configured library folders.</p>
              </div>
              <ul className="settings-view__sources">
                {(libraries ?? []).map((library) => (
                  <li key={library.id}>
                    <span>{library.name}</span>
                    <code>{library.location}</code>
                  </li>
                ))}
              </ul>
            </section>
          </div>
        ) : libraries === null ? (
          <p className="app__placeholder">Loading libraries...</p>
        ) : selectedLibrary ? (
          <div className="app-content__tracks">
            <TrackList
              libraryId={selectedLibrary.id}
              currentTrackId={current?.id ?? null}
              isPlaying={status === "playing"}
              onPlayTrack={play}
              onToggleCurrentTrack={toggle}
              refreshKey={trackRefreshVersions[selectedLibrary.id] ?? 0}
              onTracksLoaded={handleTracksLoaded}
              playCounts={trackPlayCounts[selectedLibrary.id] ?? {}}
            />
          </div>
        ) : (
          <div className="app-content__empty">
            <h2>No music yet</h2>
            <p>Add a folder in Settings, scan it, then pick a track.</p>
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

      <NowPlayingBar
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
  );
}
