import { app, BrowserWindow, ipcMain, Menu } from "electron";
import { execFileSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { installDesktopMenu } from "../desktop-menu";
import { installPlayerWindowMode } from "../install-player-window-mode";

const [page, preload, resultPath] = process.argv.slice(-3);
if (!page || !preload || !resultPath)
  throw new Error("Compact fixture paths required");
const LIBRARY_LAYOUT_HEIGHT = 600;
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
ipcMain.handle("muzo:get-update-status", () => ({
  ok: true,
  value: { state: "idle" },
}));
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

async function nativeShape(
  window: BrowserWindow,
  label: string,
): Promise<string | null> {
  if (process.platform !== "linux") return null;
  const id = String(window.getNativeWindowHandle().readUInt32LE());
  const report = execFileSync("xwininfo", ["-id", id, "-shape"], {
    encoding: "utf8",
  });
  const directory = process.env.MUZO_COMPACT_NATIVE_CAPTURE_DIR;
  if (directory) {
    await window.webContents.executeJavaScript(
      "new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))",
    );
    await new Promise((resolve) => {
      setTimeout(resolve, 100);
    });
    mkdirSync(directory, { recursive: true });
    writeFileSync(join(directory, `${label}.txt`), report);
    execFileSync("import", [
      "-window",
      "root",
      join(directory, `${label}.png`),
    ]);
  }
  return report;
}

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
  const contrastSamples = Array.from(
    root.querySelectorAll<HTMLElement>(
      ".compact-player__header,.compact-player__body,.compact-queue",
    ),
  ).map((surface) => {
    const canvas = document.createElement("canvas");
    canvas.width = 1;
    canvas.height = 1;
    const context = canvas.getContext("2d");
    if (!context) throw new Error("Colour measurement unavailable");
    context.fillStyle = getComputedStyle(surface).backgroundColor;
    context.fillRect(0, 0, 1, 1);
    const [red, green, blue, alpha] = context.getImageData(0, 0, 1, 1).data;
    const opacity = alpha / 255;
    const style = getComputedStyle(surface);
    context.clearRect(0, 0, 1, 1);
    context.fillStyle =
      style.getPropertyValue("--glass-wash").trim() || "transparent";
    context.fillRect(0, 0, 1, 1);
    const wash = context.getImageData(0, 0, 1, 1).data;
    const backgrounds = [0, 255].flatMap((backdrop) =>
      [0, wash[3] / 255].map(
        (glint) =>
          `#${[red, green, blue]
            .map((channel, index) => {
              const base = channel * opacity + backdrop * (1 - opacity);
              return Math.round(base * (1 - glint) + wash[index] * glint)
                .toString(16)
                .padStart(2, "0");
            })
            .join("")}`,
      ),
    );
    return {
      backgrounds,
      text: style.getPropertyValue("--text").trim(),
      muted: style.getPropertyValue("--text-muted").trim(),
    };
  });
  return {
    contrastSamples,
    timelineBackground: getComputedStyle(
      root.querySelector(".now-playing-bar__display") ?? root,
    ).backgroundColor,
    bottomCorners: [
      getComputedStyle(root).borderBottomLeftRadius,
      getComputedStyle(root).borderBottomRightRadius,
    ],
    width: window.innerWidth,
    height: window.innerHeight,
    overflow: document.documentElement.scrollWidth - window.innerWidth,
    controls,
    libraryHidden: document.querySelector(".app-shell")?.hasAttribute("hidden"),
  };
}
void app.whenReady().then(async () => {
  try {
    installDesktopMenu(Menu, process.platform);
    const window = new BrowserWindow({
      title: "Muzo",
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
        `new Promise((resolve,reject)=>{const deadline=Date.now()+5000;const check=()=>{if(${expression})resolve();else if(Date.now()>deadline)reject(new Error('Compact fixture timeout: '+${JSON.stringify(expression)}));else setTimeout(check,20)};check()})`,
      );
    }
    async function resize(width: number, height: number) {
      window.setContentSize(width, height);
      await until(`innerWidth === ${width} && innerHeight === ${height}`);
      await window.webContents.executeJavaScript(
        "document.fonts.ready.then(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))))",
      );
    }
    async function settleBounds(bounds: Electron.Rectangle) {
      const deadline = Date.now() + 5000;
      while (
        JSON.stringify(window.getNormalBounds()) !== JSON.stringify(bounds)
      ) {
        if (Date.now() > deadline)
          throw new Error("Native window bounds did not settle");
        await new Promise((resolve) => {
          setTimeout(resolve, 20);
        });
      }
      const [width, height] = window.getContentSize();
      await until(`innerWidth === ${width} && innerHeight === ${height}`);
    }
    const click = async (label: string) => {
      const expression = `[...document.querySelectorAll('button')].find(button => button.getAttribute('aria-label') === ${JSON.stringify(label)} || button.textContent.trim() === ${JSON.stringify(label)})`;
      await until(
        `!!(${expression}) && !(${expression}).disabled && (${expression}).getBoundingClientRect().width > 0`,
      );
      await window.webContents.executeJavaScript(
        "document.fonts.ready.then(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))))",
      );
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
    const libraryControls = [];
    for (const theme of ["aluminium", "glass", "pulse"]) {
      for (const mode of ["light", "dark"]) {
        await click("Playlists");
        await click("Play Evening");
        await until("!!document.querySelector('button[aria-label=Pause]')");
        await window.webContents.executeJavaScript(
          `document.documentElement.dataset.theme=${JSON.stringify(theme)}; document.documentElement.dataset.mode=${JSON.stringify(mode)}; document.documentElement.style.colorScheme=${JSON.stringify(mode)}`,
        );
        for (const width of [1200, 800]) {
          await resize(width, LIBRARY_LAYOUT_HEIGHT);
          libraryControls.push(
            await window.webContents.executeJavaScript(`(() => {
            const mini = document.querySelector('.app-shell button[aria-label="Mini player"]');
            const volume = document.querySelector('.app-shell input[aria-label="Volume"]');
            const rect = mini.getBoundingClientRect();
            return {width:innerWidth, top:rect.top, right:rect.right, bottom:rect.bottom, buttonWidth:rect.width, volumeTop:volume.getBoundingClientRect().top, text:mini.textContent.trim(), title:mini.title};
          })()`),
          );
          if (process.env.MUZO_COMPACT_CAPTURE_DIR) {
            mkdirSync(process.env.MUZO_COMPACT_CAPTURE_DIR, {
              recursive: true,
            });
            writeFileSync(
              join(
                process.env.MUZO_COMPACT_CAPTURE_DIR,
                `library-${theme}-${mode}-${width}.png`,
              ),
              (await window.webContents.capturePage()).toPNG(),
            );
          }
        }
        window.setBounds(original);
        await settleBounds(original);
        await click("Mini player");
        await until("!!document.querySelector('.compact-player')");
        await until(
          "!!document.querySelector('.compact-player__sleeve img')?.complete",
        );
        for (const [width, height] of [
          [420, 560],
          [360, 480],
        ]) {
          await resize(width, height);
          if (process.env.MUZO_COMPACT_CAPTURE_DIR) {
            mkdirSync(process.env.MUZO_COMPACT_CAPTURE_DIR, {
              recursive: true,
            });
            writeFileSync(
              join(
                process.env.MUZO_COMPACT_CAPTURE_DIR,
                `compact-${theme}-${mode}-solo-${width}.png`,
              ),
              (await window.webContents.capturePage()).toPNG(),
            );
          }
          const layout: unknown = await window.webContents.executeJavaScript(
            `(${measureCompact.toString()})()`,
          );
          measurements.push({
            theme,
            mode,
            queue: false,
            layout,
            nativeShape: await nativeShape(
              window,
              `${theme}-${mode}-solo-${width}`,
            ),
          });
        }
        await click("Show queue");
        await until("!!document.querySelector('.compact-queue')");
        for (const [width, height] of [
          [740, 560],
          [640, 480],
        ]) {
          await resize(width, height);
          if (process.env.MUZO_COMPACT_CAPTURE_DIR) {
            await until(
              "(()=>{const list=document.querySelector('.compact-queue__list');const bounds=list.getBoundingClientRect();return [...list.children].filter(row=>{const r=row.getBoundingClientRect();return r.bottom>bounds.top && r.top<bounds.bottom}).every(row=>row.querySelector('img')?.complete)})()",
            );
            mkdirSync(process.env.MUZO_COMPACT_CAPTURE_DIR, {
              recursive: true,
            });
            writeFileSync(
              join(
                process.env.MUZO_COMPACT_CAPTURE_DIR,
                `compact-${theme}-${mode}-queue-${width}.png`,
              ),
              (await window.webContents.capturePage()).toPNG(),
            );
          }
          const layout: unknown = await window.webContents.executeJavaScript(
            `(${measureCompact.toString()})()`,
          );
          const point = (await window.webContents.executeJavaScript(
            `(()=>{document.querySelector('.compact-player__header button').focus();const list=document.querySelector('.compact-queue__list');list.scrollTop=0;const r=list.getBoundingClientRect();return {x:Math.round(r.x+r.width/2),y:Math.round(r.y+r.height/2)}})()`,
          )) as { x: number; y: number };
          window.webContents.sendInputEvent({ type: "mouseMove", ...point });
          await window.webContents.executeJavaScript(
            "new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))",
          );
          window.webContents.sendInputEvent({
            type: "mouseWheel",
            ...point,
            deltaX: 0,
            deltaY: -300,
          });
          await until(
            "document.querySelector('.compact-queue__list').scrollTop > 0",
          );
          await window.webContents.executeJavaScript(
            "document.querySelector('.compact-queue__list button').focus()",
          );
          for (let index = 1; index < 12; index += 1) {
            window.webContents.sendInputEvent({
              type: "keyDown",
              keyCode: "Tab",
            });
            window.webContents.sendInputEvent({
              type: "keyUp",
              keyCode: "Tab",
            });
          }
          await until(
            "document.activeElement === document.querySelector('.compact-queue__list li:last-child button')",
          );
          const queueAccess: unknown =
            await window.webContents.executeJavaScript(
              `(()=>{const list=document.querySelector('.compact-queue__list');const last=list.lastElementChild.getBoundingClientRect();return {scrollTop:list.scrollTop,lastBottom:last.bottom,listBottom:list.getBoundingClientRect().bottom}})()`,
            );
          window.webContents.sendInputEvent({
            type: "keyDown",
            keyCode: "Enter",
          });
          window.webContents.sendInputEvent({ type: "char", keyCode: "\r" });
          window.webContents.sendInputEvent({
            type: "keyUp",
            keyCode: "Enter",
          });
          await until(
            "document.querySelector('.compact-queue__list li:last-child button').getAttribute('aria-current') === 'true'",
          );
          measurements.push({
            theme,
            mode,
            queue: true,
            layout,
            queueAccess,
            nativeShape: await nativeShape(
              window,
              `${theme}-${mode}-queue-${width}`,
            ),
          });
        }
        await click("Return to library");
        await until("!document.querySelector('.compact-player')");
        await settleBounds(original);
      }
    }
    await window.webContents.executeJavaScript(
      'document.documentElement.dataset.theme="glass"',
    );
    await click("Mini player");
    await click("Show queue");
    await until("!!document.querySelector('.compact-queue')");
    const fallback: unknown = await window.webContents
      .executeJavaScript(`(() => {
      const removed = [];
      for (const sheet of document.styleSheets) {
        for (let index = sheet.cssRules.length - 1; index >= 0; index -= 1) {
          const rule = sheet.cssRules[index];
          if (rule instanceof CSSSupportsRule && rule.conditionText.includes('backdrop-filter')) {
            removed.push({sheet,index,text:rule.cssText});
            sheet.deleteRule(index);
          }
        }
      }
      try {
        const canvas = document.createElement('canvas');
        canvas.width = canvas.height = 1;
        const context = canvas.getContext('2d');
        context.fillStyle = getComputedStyle(document.querySelector('.compact-queue')).backgroundColor;
        context.fillRect(0,0,1,1);
        return {queueAlpha:context.getImageData(0,0,1,1).data[3]};
      } finally {
        for (const item of removed.reverse()) item.sheet.insertRule(item.text,item.index);
      }
    })()`);
    await click("Return to library");
    await until("!document.querySelector('.compact-player')");
    await settleBounds(original);
    writeFileSync(
      resultPath,
      JSON.stringify({
        measurements,
        libraryControls,
        fallback,
        original,
        restored: window.getNormalBounds(),
        minimum: window.getMinimumSize(),
        restoredShape: await nativeShape(window, "library-restored"),
      }),
    );
    window.destroy();
  } catch (error) {
    writeFileSync(resultPath, JSON.stringify({ failure: String(error) }));
    process.stderr.write(String(error));
    process.exitCode = 1;
  } finally {
    app.quit();
  }
});
