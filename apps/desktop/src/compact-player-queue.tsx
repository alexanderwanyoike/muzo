import { ListMusic, Pause, Play } from "lucide-react";
import type { AudioPlayer } from "./useAudioPlayer";
import TrackArtwork from "./track-artwork";

export default function CompactPlayerQueue({
  player,
}: {
  player: AudioPlayer;
}) {
  return (
    <aside
      className="compact-queue"
      aria-label="Playback queue panel"
      id="compact-queue"
    >
      <header className="compact-queue__header">
        <ListMusic aria-hidden="true" />
        <div>
          <h2>Playback queue</h2>
          <p>{player.queueEntries.length} tracks</p>
        </div>
      </header>
      {player.queueEntries.length === 0 && (
        <p className="compact-queue__empty">
          Play a playlist to build a queue.
        </p>
      )}
      <ol aria-label="Playback queue" className="compact-queue__list">
        {player.queueEntries.map((entry, index) => {
          const current = player.currentQueueEntryId === entry.entryId;
          const playing = current && player.status === "playing";
          return (
            <li key={entry.entryId}>
              <button
                type="button"
                aria-current={current ? "true" : undefined}
                onClick={() => {
                  if (current) player.toggle();
                  else player.selectQueueEntry(entry.entryId);
                }}
              >
                <TrackArtwork track={entry.track} />
                <span className="compact-queue__metadata">
                  <strong>{entry.track.title}</strong>
                  <span>{entry.track.artist}</span>
                </span>
                <span className="compact-queue__position">
                  {current ? (
                    playing ? (
                      <Pause aria-label="Playing" />
                    ) : (
                      <Play aria-label="Paused" />
                    )
                  ) : (
                    index + 1
                  )}
                </span>
              </button>
            </li>
          );
        })}
      </ol>
    </aside>
  );
}
