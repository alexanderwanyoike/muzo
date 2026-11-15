import assert from "node:assert/strict";
import { copyFile, mkdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { setTimeout as delay } from "node:timers/promises";

const startupTimeoutMs = 20_000;
const requestTimeoutMs = 10_000;
const pollMs = 250;

async function connectToRenderer(profile) {
  const deadline = Date.now() + startupTimeoutMs;
  let failure;
  while (Date.now() < deadline) {
    try {
      const contents = await readFile(
        join(profile, "DevToolsActivePort"),
        "utf8",
      );
      const [port] = contents.trim().split(/\r?\n/u);
      assert.match(port, /^\d+$/u);
      const response = await fetch(`http://127.0.0.1:${port}/json/list`, {
        signal: AbortSignal.timeout(requestTimeoutMs),
      });
      const targets = await response.json();
      const page = targets.find(
        ({ type, url }) => type === "page" && url.includes("index.html"),
      );
      assert(page, "The packaged Electron renderer is still opening");
      const socket = new WebSocket(page.webSocketDebuggerUrl);
      await new Promise((resolve, reject) => {
        const timer = setTimeout(() => {
          socket.close();
          reject(new Error("The renderer debugger did not connect"));
        }, requestTimeoutMs);
        socket.addEventListener(
          "open",
          () => {
            clearTimeout(timer);
            resolve();
          },
          { once: true },
        );
        socket.addEventListener(
          "error",
          () => {
            clearTimeout(timer);
            reject(new Error("The renderer debugger did not connect"));
          },
          { once: true },
        );
      });
      return socket;
    } catch (error) {
      failure = error;
      await delay(pollMs);
    }
  }
  throw new Error("The packaged Electron renderer did not become available", {
    cause: failure,
  });
}

function renderer(socket) {
  let sequence = 0;
  return (expression) =>
    new Promise((resolve, reject) => {
      const id = ++sequence;
      const timer = setTimeout(() => {
        socket.removeEventListener("message", receive);
        reject(new Error("The renderer evaluation timed out"));
      }, requestTimeoutMs);
      function receive({ data }) {
        const message = JSON.parse(data);
        if (message.id !== id) return;
        clearTimeout(timer);
        socket.removeEventListener("message", receive);
        if (message.error || message.result?.exceptionDetails) {
          reject(new Error(`Renderer evaluation failed: ${data}`));
        } else {
          resolve(message.result?.result?.value);
        }
      }
      socket.addEventListener("message", receive);
      socket.send(
        JSON.stringify({
          id,
          method: "Runtime.evaluate",
          params: { expression, awaitPromise: true, returnByValue: true },
        }),
      );
    });
}

const [profile, folder, report] = process.argv.slice(2);
assert(
  profile && folder && report,
  "Expected the Chromium profile, audio directory, and report path",
);
await mkdir(folder);
await copyFile(
  new URL("../fixtures/tone.mp3", import.meta.url),
  join(folder, "Tone.mp3"),
);
await copyFile(
  new URL("../fixtures/embedded-cover.mp3", import.meta.url),
  join(folder, "Artwork.mp3"),
);

const socket = await connectToRenderer(profile);
const evaluate = renderer(socket);
try {
  const deadline = Date.now() + startupTimeoutMs;
  let ready = false;
  while (Date.now() < deadline) {
    ready = await evaluate(
      "Boolean(window.__MUZO_RUNTIME__ && document.querySelector('.app-sidebar__nav'))",
    );
    if (ready) break;
    await delay(pollMs);
  }
  assert(ready, "The installed Electron library did not become ready");

  const counts = await evaluate(`(async () => {
    const command = window.__MUZO_RUNTIME__.invoke;
    const library = await command("add_library", {
      input: {name: "Holloway Sessions", kind: "filesystem", location: ${JSON.stringify(folder)}},
    });
    const tracks = await command("list_tracks", {libraryId: library.id});
    if (tracks.length !== 2) throw new Error("Both audio files must be indexed");
    const [first, second] = tracks;
    await command("edit_track_metadata", {input: {
      libraryId: library.id, trackId: first.id, title: "Holloway edit",
      artist: "Mara Vale", album: "Field Notes", trackNumber: 3, discNumber: 1,
      genre: "Ambient", year: 2024,
    }});
    const playlist = await command("create_playlist", {input: {name: "Evening Walk"}});
    for (const track of [first, second, first]) {
      await command("add_track_to_playlist", {input: {playlistId: playlist.id, trackId: track.id}});
    }
    for (const track of [first, first, second]) {
      await command("record_track_play", {input: {libraryId: library.id, trackId: track.id}});
    }
    const playlists = await command("list_playlists");
    const updatedTracks = await command("list_tracks", {libraryId: library.id});
    const history = await command("list_track_play_counts", {input: {libraryId: library.id}});
    return {
      tracks: tracks.length,
      orderedEntries: playlists[0].entries.map(({trackId}) => trackId),
      expectedEntries: [first.id, second.id, first.id],
      edit: updatedTracks.find(({id}) => id === first.id)?.title,
      plays: history.reduce((sum, row) => sum + row.playCount, 0),
    };
  })()`);
  assert.equal(counts.tracks, 2);
  assert.deepStrictEqual(counts.orderedEntries, counts.expectedEntries);
  assert.equal(counts.edit, "Holloway edit");
  assert.equal(counts.plays, 3);

  const preferences = await evaluate(`(() => {
    const settings = [...document.querySelectorAll(".app-sidebar__nav button")]
      .find(button => button.textContent.includes("Settings"));
    settings.click();
    return true;
  })()`);
  assert(preferences);
  await evaluate("document.querySelector('input[aria-label=Walnut]').click()");
  await evaluate(
    "Array.from(document.querySelectorAll('label')).find(label => label.textContent.trim() === 'Dark').querySelector('input[name=mode]').click()",
  );
  await evaluate(
    `document.querySelector('input[aria-label="Glow cyan"]').click()`,
  );
  const sidebarResizeSteps = 9;
  for (let step = 0; step < sidebarResizeSteps; step += 1) {
    await evaluate(
      "document.querySelector('[role=separator]').dispatchEvent(new KeyboardEvent('keydown', {key: 'ArrowRight', bubbles: true}))",
    );
  }
  const saved = await evaluate(`({
    appearance: JSON.parse(localStorage.getItem("muzo.appearance.v1")),
    sidebar: JSON.parse(localStorage.getItem("muzo.sidebar-width.v1")),
  })`);
  assert.deepStrictEqual(saved, {
    appearance: { theme: "walnut", mode: "dark", accent: "glow-cyan" },
    sidebar: 312,
  });
  await writeFile(report, JSON.stringify(saved));
  console.log(
    "The installed Electron app saved the upgrade fixture and preferences.",
  );
} finally {
  socket.close();
}
