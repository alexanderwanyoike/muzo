import { useState } from "react";
import { createRoot } from "react-dom/client";
import NowPlayingBar from "../../src/NowPlayingBar";
import { applyAppearance } from "../../src/appearance";
import "../../src/themes.css";
import "../../src/styles.css";

applyAppearance({ theme: "aluminium", mode: "dark", accent: "cobalt" });
window.__MUZO_RUNTIME__ = {
  async invoke<T>(): Promise<T> {
    return { ok: true, value: null } as T;
  },
  async openDirectory() {
    return null;
  },
};
function TransportFixture() {
  const [position, setPosition] = useState(30);
  return (
    <NowPlayingBar
      current={{
        id: "demo",
        libraryId: "demo",
        title: "Demo track",
        artist: "Demo artist",
        album: null,
        trackNumber: 1,
        discNumber: 1,
        genre: null,
        year: null,
        metadataOverridden: false,
        durationSeconds: 180,
        filePath: "/Music/demo.flac",
      }}
      status="paused"
      positionSeconds={position}
      durationSeconds={180}
      volume={1}
      playbackError={null}
      onToggle={() => {}}
      onSeek={setPosition}
      onSetVolume={() => {}}
    />
  );
}
const root = document.getElementById("root");
if (!root) throw new Error("Transport fixture root required");
createRoot(root).render(<TransportFixture />);
