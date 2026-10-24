import { useEffect, useRef } from "react";
import { Maximize2, ListMusic } from "lucide-react";
import CompactPlayerQueue from "./compact-player-queue";
import TrackArtwork from "./track-artwork";
import NowPlayingBar from "./NowPlayingBar";
import type { AudioPlayer } from "./useAudioPlayer";
import "./compact-player.css";

interface Props {
  player: AudioPlayer;
  artworkRefreshKey: number;
  pending: boolean;
  error: string | null;
  onRestore: () => void;
  queueOpen: boolean;
  onToggleQueue: () => void;
}

export default function CompactPlayer({
  player,
  artworkRefreshKey,
  pending,
  error,
  onRestore,
  queueOpen,
  onToggleQueue,
}: Props) {
  const restoreButton = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    restoreButton.current?.focus();
  }, []);
  return (
    <section
      className={`compact-player${queueOpen ? " compact-player--queue" : ""}`}
      aria-label="Mini player"
    >
      {player.current && (
        <div className="compact-player__atmosphere" aria-hidden="true">
          <TrackArtwork
            track={player.current}
            eager
            refreshKey={artworkRefreshKey}
          />
        </div>
      )}
      <header className="compact-player__header">
        <span>Muzo</span>
        <div className="compact-player__window-actions">
          <button
            type="button"
            aria-label={queueOpen ? "Hide queue" : "Show queue"}
            title={queueOpen ? "Hide queue" : "Show queue"}
            aria-expanded={queueOpen}
            aria-controls="compact-queue"
            disabled={pending}
            onClick={onToggleQueue}
          >
            <ListMusic aria-hidden="true" />
          </button>
          <button
            type="button"
            ref={restoreButton}
            aria-label="Return to library"
            title="Return to library"
            disabled={pending}
            onClick={onRestore}
          >
            <Maximize2 aria-hidden="true" />
          </button>
        </div>
      </header>
      <div className="compact-player__workspace">
        {queueOpen && <CompactPlayerQueue player={player} />}
        <div className="compact-player__body">
          <figure
            className="compact-player__sleeve"
            aria-label="Current album artwork"
          >
            {player.current ? (
              <TrackArtwork
                track={player.current}
                eager
                refreshKey={artworkRefreshKey}
              />
            ) : (
              <div className="compact-player__empty">
                Choose some music in your library
              </div>
            )}
          </figure>
          <div className="compact-player__metadata">
            <h1>{player.current?.title ?? "Not playing"}</h1>
            <p>{player.current?.artist ?? "Choose a track in your library"}</p>
            {player.current?.album && (
              <p className="compact-player__album">{player.current.album}</p>
            )}
          </div>
          <NowPlayingBar
            current={player.current}
            status={player.status}
            positionSeconds={player.positionSeconds}
            durationSeconds={player.durationSeconds}
            volume={player.volume}
            playbackError={player.playbackError}
            hasNext={player.hasNext}
            hasPrevious={player.hasPrevious}
            artworkRefreshKey={artworkRefreshKey}
            onNext={player.next}
            onPrevious={player.previous}
            onToggle={player.toggle}
            onSeek={player.seek}
            onSetVolume={player.setVolume}
          />
          {player.playbackError && (
            <p role="alert" className="compact-player__playback-error">
              {player.playbackError}
            </p>
          )}
        </div>
      </div>
      {error && (
        <p role="alert" className="compact-player__error">
          {error}
        </p>
      )}
    </section>
  );
}
