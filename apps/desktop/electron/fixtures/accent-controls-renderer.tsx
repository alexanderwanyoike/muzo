import { createRoot } from "react-dom/client";
import NowPlayingBar from "../../src/NowPlayingBar";
import "@fontsource-variable/manrope";
import "../../src/themes.css";
import "../../src/styles.css";
import "../../src/compact-player.css";

const root = document.getElementById("root");
if (!root) throw new Error("Accent fixture root required");
createRoot(root).render(
  <div id="stage" className="app-shell">
    <NowPlayingBar
      current={{
        id: "fixture",
        libraryId: "fixture",
        title: "First light",
        artist: "Linden",
        album: null,
        trackNumber: null,
        discNumber: null,
        genre: null,
        year: null,
        metadataOverridden: false,
        durationSeconds: 240,
        filePath: "/fixture.flac",
      }}
      status="playing"
      positionSeconds={90}
      durationSeconds={240}
      volume={0.7}
      playbackError={null}
      onToggle={() => {}}
      onSeek={() => {}}
      onSetVolume={() => {}}
    />
  </div>,
);
