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
  return (
    <section
      className={`compact-player${queueOpen ? " compact-player--queue" : ""}`}
      aria-label="Mini player"
    >
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
