import { app, BrowserWindow, desktopCapturer, ipcMain, Menu, nativeTheme, screen } from "electron";
import { execFile, spawn, type ChildProcess } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { release } from "node:os";
import { join } from "node:path";
import { promisify } from "node:util";
import { createPlayerWindow } from "../create-player-window";
import { installDesktopGlass } from "../install-desktop-glass";
import { installPlayerWindowMode } from "../install-player-window-mode";
import { installDesktopMenu } from "../desktop-menu";

const [page, preload, resultPath, icon] = process.argv.slice(-4);
if (!page || !preload || !resultPath || !icon) throw new Error("Desktop glass fixture paths required");
if (process.platform === "linux") app.commandLine.appendSwitch("ozone-platform", "x11");
let compositor: ChildProcess | null = null;
let window: BrowserWindow | null = null;
let backdrop: BrowserWindow | null = null;
let artwork: string | null = null;
const track = { id: "first", libraryId: "music", title: "First light", artist: "Linden", album: "North Window", trackNumber: 1, discNumber: 1, genre: null, year: null, metadataOverridden: false, durationSeconds: 240, filePath: "/Music/demo.flac" };
ipcMain.handle("muzo:invoke", (_event, command: string, args: unknown) => {
  if (command === "fixture_set_artwork" && args && typeof args === "object" && "uri" in args && typeof args.uri === "string") artwork = args.uri;
  let value: unknown = [];
  if (command === "list_libraries") value = [{ id: "music", name: "Evening listening", kind: "filesystem", location: "/Music" }];
  if (command === "list_tracks") value = [track];
  if (command === "get_track_artwork") value = { ok: true, value: artwork };
  if (command === "prepare_track_audio_source") value = { url: "fixture-audio", mimeType: "audio/flac" };
  return { ok: true, value };
});
installPlayerWindowMode(ipcMain, (event) => BrowserWindow.fromWebContents(event.sender));
installDesktopGlass(ipcMain, (event) => BrowserWindow.fromWebContents(event.sender), () => ({ platform: process.platform, release: release(), reducedTransparency: false }));

async function until(expression: string) {
  if (!window) throw new Error("Player unavailable");
  await window.webContents.executeJavaScript(`new Promise((resolve,reject)=>{const deadline=Date.now()+5000;const check=()=>{if(${expression})resolve();else if(Date.now()>deadline)reject(new Error('Glass fixture timeout: '+${JSON.stringify(expression)}));else setTimeout(check,20)};check()})`);
}
async function click(label: string) {
  await until(`!![...document.querySelectorAll('button,input')].find(element=>element.getAttribute('aria-label')===${JSON.stringify(label)} || element.textContent?.trim()===${JSON.stringify(label)} || (element.tagName==='INPUT' && [...element.labels].some(label=>label.textContent.trim()===${JSON.stringify(label)})))`);
  await window?.webContents.executeJavaScript(`[...document.querySelectorAll('button,input')].find(element=>element.getAttribute('aria-label')===${JSON.stringify(label)} || element.textContent?.trim()===${JSON.stringify(label)} || (element.tagName==='INPUT' && [...element.labels].some(label=>label.textContent.trim()===${JSON.stringify(label)}))).click()`);
}
async function settle() {
  await window?.webContents.executeJavaScript("document.fonts.ready.then(()=>Promise.all(document.getAnimations().map(animation=>animation.finished))).then(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))))");
}
async function point() {
  return window?.webContents.executeJavaScript(`(()=>{const element=document.querySelector('.compact-player__body') ?? document.querySelector('.app-content__tracks');const r=element.getBoundingClientRect();const style=getComputedStyle(element);return {x:Math.round(r.right-15),y:Math.round(r.bottom-15),text:style.getPropertyValue('--text').trim(),muted:style.getPropertyValue('--text-muted').trim()}})()`) as Promise<{x:number;y:number;text:string;muted:string}>;
}
async function sample() {
  if (!window) throw new Error("Player unavailable");
  await settle();
  const location = await point();
  const image = await window.webContents.capturePage();
  const bitmap = image.toBitmap();
  const offset = (location.y * image.getSize().width + location.x) * 4;
  return { alpha: bitmap[offset + 3], colour: [bitmap[offset + 2], bitmap[offset + 1], bitmap[offset]], text: location.text, muted: location.muted };
}
async function desktop() {
  const display = screen.getPrimaryDisplay();
  const sources = await desktopCapturer.getSources({ types: ["screen"], thumbnailSize: display.size });
  const source = sources.find(({display_id: id}) => id === String(display.id));
  if (!source) throw new Error("Native desktop image unavailable");
  return source.thumbnail;
}
async function capture(name: string) {
  const directory = process.env.MUZO_GLASS_CAPTURE_DIR;
  if (!directory || !window) return;
  mkdirSync(directory, { recursive: true });
  const image = process.platform === "linux" ? await desktop() : await window.webContents.capturePage();
  writeFileSync(join(directory, `${name}.png`), image.toPNG());
}
void app.whenReady().then(async () => {
  try {
    if (process.platform === "linux") {
      compositor = spawn("metacity", ["--composite", "--sm-disable"], { stdio: "ignore" });
      const deadline = Date.now() + 5000;
      let ready = false;
      while (!ready) {
        const probe = await promisify(execFile)("xprop", ["-root", "_NET_SUPPORTING_WM_CHECK"]);
        ready = probe.stdout.includes("window id #");
        if (Date.now() > deadline) throw new Error("Native compositor unavailable");
        if (!ready) await new Promise((resolve) => { setTimeout(resolve, 20); });
      }
      backdrop = new BrowserWindow({x: 0, y: 0, width: 1600, height: 1100, frame: false, focusable: false, backgroundColor: "#6689b0"});
      await backdrop.loadURL(`data:text/html,${encodeURIComponent('<body style="margin:0;background:#6689b0;height:100vh;color:white;font:22px sans-serif"><div style="padding:20px">Desktop behind Muzo</div></body>')}`);
    }
    installDesktopMenu(Menu, process.platform);
    window = createPlayerWindow(icon, preload);
    window.webContents.setBackgroundThrottling(false);
    window.setBounds({ x: 140, y: 100, width: 1200, height: 800 });
    await window.loadFile(page);
    window.focus();
    await click("Show tracks for Evening listening");
    await click("Play First light");
    await until("!!document.querySelector('button[aria-label=Pause]')");
    const originalSize = window.getSize();
    const samples = [];
    for (const mode of ["light", "dark"]) {
      await click("Settings");
      await click("Frosted glass");
      await click(mode === "light" ? "Light" : "Dark");
      await until("document.documentElement.dataset.desktopGlass === 'true'");
      await click("Songs");
      samples.push({mode, view: "library", ...await sample()});
      await capture(`glass-${mode}-library`);
      await click("Mini player");
      await until("!!document.querySelector('.compact-player')");
      await until("!!document.querySelector('.compact-player__sleeve img')?.complete");
      samples.push({mode, view: "mini", ...await sample()});
      await capture(`glass-${mode}-mini`);
      window.setContentSize(460, 580);
      samples.push({mode, view: "resized", ...await sample()});
      await click("Return to library");
      await until("!document.querySelector('.compact-player')");
    }
    let backdropDifference = 0;
    if (backdrop) {
      await settle();
      const location = await point();
      const bounds = window.getContentBounds();
      const before = await desktop();
      await backdrop.webContents.executeJavaScript("document.body.style.background='#b6a690'");
      await new Promise((resolve) => { setTimeout(resolve, 150); });
      const after = await desktop();
      const offset = ((bounds.y+location.y)*after.getSize().width+bounds.x+location.x)*4;
      const first = before.toBitmap();
      const second = after.toBitmap();
      backdropDifference = [0,1,2].reduce((sum,channel)=>sum+Math.abs(first[offset+channel]-second[offset+channel]),0);
      await capture("glass-dark-library-warm-desktop");
    }
    window.webContents.debugger.attach("1.3");
    await window.webContents.debugger.sendCommand("Emulation.setEmulatedMedia", {features:[{name:"prefers-reduced-transparency",value:"reduce"}]});
    const reducedAlpha = (await sample()).alpha;
    window.webContents.debugger.detach();
    await click("Settings");
    await click("Studio");
    await until("!document.documentElement.dataset.desktopGlass");
    await click("Songs");
    const restoredAlpha = (await sample()).alpha;
    writeFileSync(resultPath,JSON.stringify({samples,restoredAlpha,reducedAlpha,restoredSize:window.getSize(),originalSize,backdropDifference}));
  } catch(error) {
    writeFileSync(resultPath,JSON.stringify({failure:String(error)}));
  } finally {
    window?.destroy();
    backdrop?.destroy();
    compositor?.kill();
    app.quit();
  }
});
