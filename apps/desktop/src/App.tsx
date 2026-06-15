import { useEffect, useState } from "react";
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
  const [scanningLibraryId, setScanningLibraryId] = useState<string | null>(null);
  const [scanMessage, setScanMessage] = useState<string | null>(null);
  const [selectedLibraryId, setSelectedLibraryId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = async () => {
    setError(null);
    try {
      const libs = await listLibraries();
      setLibraries(libs);
      const counts: Record<string, number> = {};
      for (const lib of libs) {
        try {
          const tracks = await listTracks(lib.id);
          counts[lib.id] = tracks.length;
        } catch {
          counts[lib.id] = 0;
        }
      }
      setTrackCounts(counts);
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

  return (
    <main className="app app--has-player">
      <header className="app__header">
        <h1>Muzo</h1>
        <p className="app__tagline">Your music, your libraries.</p>
      </header>

      <section className="app__section">
        <h2>Add a library</h2>
        <AddLibraryForm onAdded={refresh} />
      </section>

      <section className="app__section">
        <h2>Libraries</h2>
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
          <p className="app__placeholder">Loading...</p>
        ) : (
          <LibraryList
            libraries={libraries}
            trackCounts={trackCounts}
            scanningLibraryId={scanningLibraryId}
            selectedLibraryId={selectedLibraryId}
            onScan={handleScan}
            onSelect={handleSelect}
            renderTracks={(libraryId) => (
              <TrackList
                libraryId={libraryId}
                currentTrackId={current?.id ?? null}
                isPlaying={status === "playing"}
                onPlayTrack={play}
              />
            )}
          />
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
