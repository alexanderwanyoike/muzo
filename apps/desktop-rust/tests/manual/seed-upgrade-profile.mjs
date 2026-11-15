import assert from "node:assert/strict";
import { copyFile, mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { connectToRenderer, renderer } from "./renderer-debugger.mjs";

const startupTimeoutMs = 20_000;
const pollMs = 250;
const audioFixtures = [
  ["tone.mp3", "Tone.mp3"],
  ["embedded-cover.mp3", "Artwork.mp3"],
];
const resizedSidebarWidth = 312;

const [profile, folder, report] = process.argv.slice(2);
assert(
  profile && folder && report,
  "Expected the Chromium profile, audio directory, and report path",
);
await mkdir(folder);
for (const [fixture, name] of audioFixtures) {
  await copyFile(
    new URL(`../fixtures/${fixture}`, import.meta.url),
    join(folder, name),
  );
}

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
    if (tracks.length !== ${audioFixtures.length}) throw new Error("Both audio files must be indexed");
    const [first, second] = tracks;
    await command("edit_track_metadata", {input: {
      libraryId: library.id, trackId: first.id, title: "Holloway edit",
      artist: "Mara Vale", album: "Field Notes", trackNumber: 3, discNumber: 1,
      genre: "Ambient", year: 2024,
    }});
    const playlist = await command("create_playlist", {input: {name: "Evening Walk"}});
    const orderedTracks = [first, second, first];
    for (const track of orderedTracks) {
      await command("add_track_to_playlist", {input: {playlistId: playlist.id, trackId: track.id}});
      await command("record_track_play", {input: {libraryId: library.id, trackId: track.id}});
    }
    const playlists = await command("list_playlists");
    const updatedTracks = await command("list_tracks", {libraryId: library.id});
    const history = await command("list_track_play_counts", {input: {libraryId: library.id}});
    return {
      orderedEntries: playlists[0].entries.map(({trackId}) => trackId),
      expectedEntries: orderedTracks.map(({id}) => id),
      edit: updatedTracks.find(({id}) => id === first.id)?.title,
      plays: history.reduce((sum, row) => sum + row.playCount, 0),
    };
  })()`);
  assert.deepStrictEqual(counts.orderedEntries, counts.expectedEntries);
  assert.equal(counts.edit, "Holloway edit");
  assert.equal(counts.plays, counts.expectedEntries.length);

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
    sidebar: resizedSidebarWidth,
  });
  await writeFile(report, JSON.stringify(saved));
  console.log(
    "The installed Electron app saved the upgrade fixture and preferences.",
  );
} finally {
  socket.close();
}
