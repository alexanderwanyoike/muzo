import { useCallback, useEffect, useState } from "react";
import AddLibraryForm from "./AddLibraryForm";
import LibraryList from "./LibraryList";
import TrackList from "./TrackList";
import NowPlayingBar from "./NowPlayingBar";
import { useAudioPlayer } from "./useAudioPlayer";
import { listLibraries, listTracks, scanLibrary } from "./api";
import type { LibraryDto } from "./types";

interface ListError {
  kind: string;
  message?: string;
}

export default function App() {
  const {
    current,
    status,
    positionSeconds,
    durationSeconds,
    volume,
    play,
    toggle,
    seek,
    setVolume,
  } = useAudioPlayer();

  const [libraries, setLibraries] = useState<LibraryDto[] | null>(null);
  const [trackCounts, setTrackCounts] = useState<Record<string, number>>({});
  const [trackRefreshVersions, setTrackRefreshVersions] = useState<Record<string, number>>({});
  const [scanningLibraryId, setScanningLibraryId] = useState<string | null>(null);
  const [scanMessage, setScanMessage] = useState<string | null>(null);
  const [selectedLibraryId, setSelectedLibraryId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

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
  }

  function handleLibraryAdded(library: LibraryDto) {
    setLibraries((prev) => (prev ? [...prev, library] : [library]));
    setSelectedLibraryId(library.id);
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

        <section className="app-sidebar__section">
          <h2>Library</h2>
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

        <section className="app-sidebar__section app-sidebar__section--add">
          <h2>Add folder</h2>
          <AddLibraryForm onAdded={handleLibraryAdded} />
        </section>
      </aside>

      <section className="app-content">
        <header className="app-content__toolbar">
          <div>
            <p className="app-content__eyebrow">Now browsing</p>
            <h2>{selectedLibrary?.name ?? "No library selected"}</h2>
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
        {libraries === null ? (
          <p className="app__placeholder">Loading libraries...</p>
        ) : selectedLibrary ? (
          <div className="app-content__tracks">
            <TrackList
              libraryId={selectedLibrary.id}
              currentTrackId={current?.id ?? null}
              isPlaying={status === "playing"}
              onPlayTrack={play}
              refreshKey={trackRefreshVersions[selectedLibrary.id] ?? 0}
              onTracksLoaded={handleTracksLoaded}
            />
          </div>
        ) : (
          <div className="app-content__empty">
            <h2>No music yet</h2>
            <p>Add a folder from the sidebar, scan it, then pick a track.</p>
          </div>
        )}
      </section>

      <NowPlayingBar
        current={current}
        status={status}
        positionSeconds={positionSeconds}
        durationSeconds={durationSeconds}
        volume={volume}
        onToggle={toggle}
        onSeek={seek}
        onSetVolume={setVolume}
      />
    </main>
  );
}
