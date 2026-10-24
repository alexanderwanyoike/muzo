import { app, BrowserWindow, ipcMain } from "electron";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { installPlayerWindowMode } from "../install-player-window-mode";

const [page, preload, resultPath] = process.argv.slice(-3);
if (!page || !preload || !resultPath)
  throw new Error("Compact fixture paths required");
const track = {
  id: "first",
  libraryId: "music",
  title: "A long title for a quiet evening recording",
  artist: "Linden",
  album: "North Window",
  trackNumber: 1,
  discNumber: 1,
  genre: null,
  year: null,
  metadataOverridden: false,
  durationSeconds: 240,
  filePath: "/Music/demo.flac",
};
let artwork: string | null = null;
ipcMain.handle("muzo:invoke", (_event, command: string, args: unknown) => {
  if (
    command === "fixture_set_artwork" &&
    args &&
    typeof args === "object" &&
    "uri" in args &&
    typeof args.uri === "string"
  )
    artwork = args.uri;
  let value: unknown = [];
  if (command === "list_libraries")
    value = [
      { id: "music", name: "Music", kind: "filesystem", location: "/Music" },
    ];
  if (command === "list_tracks") value = [track];
  if (command === "list_playlists")
    value = [
      {
        id: "evening",
        name: "Evening",
        entries: [
          "First light",
          "A little further",
          "North window",
          "Still water",
          "After the rain",
          "Blue hour",
          "Paper moon",
          "Soft geometry",
          "Field notes",
          "A place to return",
          "The long way home",
          "Between the lines",
        ].map((title, index) => ({
          id: `entry-${index}`,
          trackId: `track-${index}`,
          trackTitle: title,
          trackArtist: "Linden",
          trackDurationSeconds: 240,
          trackLibraryId: "music",
          position: index,
        })),
      },
    ];
  if (command === "prepare_track_audio_source")
    value = { url: "fixture-audio", mimeType: "audio/flac" };
  if (command === "get_track_artwork") value = { ok: true, value: artwork };
  return { ok: true, value };
});
installPlayerWindowMode(ipcMain, (event) =>
  BrowserWindow.fromWebContents(event.sender),
);

function measureCompact() {
  const root = document.querySelector<HTMLElement>(".compact-player");
  if (!root) throw new Error("Mini player missing");
  const controls = Array.from(
    root.querySelectorAll<HTMLElement>("button,input"),
  )
    .filter(
      (element) =>
        getComputedStyle(element).display !== "none" &&
        !element.closest(".compact-queue__list"),
    )
    .map((element) => {
      const rect = element.getBoundingClientRect();
      return {
        name: element.getAttribute("aria-label"),
        left: rect.left,
        right: rect.right,
        top: rect.top,
        bottom: rect.bottom,
      };
    });
  return {
    width: window.innerWidth,
    height: window.innerHeight,
    overflow: document.documentElement.scrollWidth - window.innerWidth,
    controls,
    libraryHidden: document.querySelector(".app-shell")?.hasAttribute("hidden"),
  };
}
void app.whenReady().then(async () => {
  try {
    const window = new BrowserWindow({
      width: 1200,
      height: 800,
      minWidth: 800,
      minHeight: 600,
      show: true,
      webPreferences: {
        preload,
        contextIsolation: true,
        nodeIntegration: false,
        backgroundThrottling: false,
      },
    });
    await window.loadFile(page);
    async function until(expression: string) {
      await window.webContents.executeJavaScript(
        `new Promise((resolve,reject)=>{const deadline=Date.now()+5000;const check=()=>{if(${expression})resolve();else if(Date.now()>deadline)reject(new Error('Compact fixture timeout'));else setTimeout(check,20)};check()})`,
      );
    }
    const click = async (label: string) => {
      const expression = `[...document.querySelectorAll('button')].find(button => button.getAttribute('aria-label') === ${JSON.stringify(label)} || button.textContent.trim() === ${JSON.stringify(label)})`;
      await until(`!!(${expression})`);
      const point = (await window.webContents.executeJavaScript(
        `(()=>{const rect=(${expression}).getBoundingClientRect();return {x:Math.round(rect.x+rect.width/2),y:Math.round(rect.y+rect.height/2)}})()`,
      )) as { x: number; y: number };
      window.webContents.focus();
      window.webContents.sendInputEvent({
        type: "mouseDown",
        button: "left",
        clickCount: 1,
        ...point,
      });
      window.webContents.sendInputEvent({
        type: "mouseUp",
        button: "left",
        clickCount: 1,
        ...point,
      });
    };
    await click("Show tracks for Music");
    await click(`Play ${track.title}`);
    await until("!!document.querySelector('button[aria-label=Pause]')");
    const original = window.getNormalBounds();
    const measurements = [];
    for (const mode of ["light", "dark"]) {
      if (mode === "dark") {
        await click("Playlists");
        await click("Play Evening");
        await until("!!document.querySelector('button[aria-label=Pause]')");
      }
      await window.webContents.executeJavaScript(
        `document.documentElement.dataset.mode=${JSON.stringify(mode)}`,
      );
      await click("Mini player");
      await until("!!document.querySelector('.compact-player')");
      await until(
        "!!document.querySelector('.compact-player__sleeve img')?.complete",
      );
      for (const [width, height] of [
        [420, 560],
        [360, 480],
      ]) {
        window.setContentSize(width, height);
        await window.webContents.executeJavaScript(
          "new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))",
        );
        if (process.env.MUZO_COMPACT_CAPTURE_DIR) {
          mkdirSync(process.env.MUZO_COMPACT_CAPTURE_DIR, { recursive: true });
          writeFileSync(
            join(
              process.env.MUZO_COMPACT_CAPTURE_DIR,
              `compact-${mode}-solo-${width}.png`,
            ),
            (await window.webContents.capturePage()).toPNG(),
          );
        }
        const layout: unknown = await window.webContents.executeJavaScript(
          `(${measureCompact.toString()})()`,
        );
        measurements.push({
          mode,
          queue: false,
          layout,
        });
      }
      await click("Show queue");
      await until("!!document.querySelector('.compact-queue')");
      for (const [width, height] of [
        [740, 560],
        [640, 480],
      ]) {
        window.setContentSize(width, height);
        await window.webContents.executeJavaScript(
          "new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))",
        );
        if (process.env.MUZO_COMPACT_CAPTURE_DIR) {
          mkdirSync(process.env.MUZO_COMPACT_CAPTURE_DIR, { recursive: true });
          writeFileSync(
            join(
              process.env.MUZO_COMPACT_CAPTURE_DIR,
              `compact-${mode}-queue-${width}.png`,
            ),
            (await window.webContents.capturePage()).toPNG(),
          );
        }
        const layout: unknown = await window.webContents.executeJavaScript(
          `(${measureCompact.toString()})()`,
        );
        measurements.push({
          mode,
          queue: true,
          layout,
        });
      }
      await click("Return to library");
      await until("!document.querySelector('.compact-player')");
    }
    writeFileSync(
      resultPath,
      JSON.stringify({
        measurements,
        original,
        restored: window.getNormalBounds(),
        minimum: window.getMinimumSize(),
      }),
    );
    window.destroy();
  } catch (error) {
    process.stderr.write(String(error));
    process.exitCode = 1;
  } finally {
    app.quit();
  }
});
