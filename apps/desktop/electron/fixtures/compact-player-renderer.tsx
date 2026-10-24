import { createRoot } from "react-dom/client";
import App from "../../src/App";
import { invokeCommand } from "../../src/runtime-bridge";
import { applyAppearance } from "../../src/appearance";
import "@fontsource-variable/manrope";
import "../../src/themes.css";
import "../../src/styles.css";

class FixtureAudio extends EventTarget {
  src = "";

  currentTime = 0;

  duration = 240;

  volume = 1;

  paused = true;

  load() {
    this.dispatchEvent(new Event("durationchange"));
  }

  async play() {
    this.paused = false;
    this.dispatchEvent(new Event("playing"));
  }

  pause() {
    this.paused = true;
    this.dispatchEvent(new Event("pause"));
  }
}
Object.defineProperty(window, "Audio", { value: FixtureAudio });
applyAppearance({ theme: "aluminium", mode: "dark", accent: "cobalt" });
async function renderFixture() {
  const cover = new Image();
  cover.src = `data:image/svg+xml;base64,${btoa('<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512"><defs><linearGradient id="sky" x2="1" y2="1"><stop stop-color="#789ca2"/><stop offset="1" stop-color="#1d2c3c"/></linearGradient></defs><rect width="512" height="512" fill="url(#sky)"/><circle cx="270" cy="230" r="132" fill="none" stroke="#e5ecdf" stroke-opacity=".45"/><path d="M0 370Q160 200 320 365T640 365V512H0" fill="#182834" fill-opacity=".65"/><path d="M0 410Q160 260 320 410T640 410" fill="none" stroke="#d6e6e4" stroke-opacity=".5"/><text x="40" y="65" font-family="sans-serif" font-size="16" fill="#edf3ec" letter-spacing="5">NORTH WINDOW</text></svg>')}`;
  await cover.decode();
  const canvas = document.createElement("canvas");
  canvas.width = 512;
  canvas.height = 512;
  const context = canvas.getContext("2d");
  if (!context) throw new Error("Fixture artwork canvas unavailable");
  context.drawImage(cover, 0, 0);
  await invokeCommand("fixture_set_artwork", {
    uri: canvas.toDataURL("image/png"),
  });
  const root = document.getElementById("root");
  if (!root) throw new Error("Compact fixture root required");
  createRoot(root).render(<App />);
}
void renderFixture();
