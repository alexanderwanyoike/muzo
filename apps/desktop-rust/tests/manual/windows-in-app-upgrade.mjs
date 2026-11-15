import assert from "node:assert/strict";
import { execFile, spawn } from "node:child_process";
import { access, mkdir, readFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

import { startLegacyUpdateFeed } from "./legacy-update-feed.mjs";
import {
  captureLegacyProfile,
  verifyProfileUpgrade,
} from "./profile-upgrade.mjs";
import { connectToRenderer, renderer } from "./renderer-debugger.mjs";

const execute = promisify(execFile);
const pollMs = 250;
const downloadTimeoutMs = 45_000;
const restartTimeoutMs = 40_000;
const rendererButton = "document.querySelector('.update-notice button')";

async function waitFor(check, timeoutMs, description) {
  const deadline = Date.now() + timeoutMs;
  let failure;
  while (Date.now() < deadline) {
    try {
      return await check();
    } catch (error) {
      failure = error;
      await delay(pollMs);
    }
  }
  throw new Error(description, { cause: failure });
}

async function waitForUpdateButton(evaluate, label) {
  await waitFor(
    async () => {
      const button = await evaluate(
        rendererButton +
          " && { label: " +
          rendererButton +
          ".getAttribute('aria-label'), disabled: " +
          rendererButton +
          ".disabled }",
      );
      assert.deepEqual(button, { label, disabled: false });
    },
    downloadTimeoutMs,
    "The installed updater did not show " + label,
  );
}

const [legacyExecutable, candidate, manifest, fixture] = process.argv
  .slice(2)
  .map((path) => resolve(path));
assert(legacyExecutable && candidate && manifest && fixture);
assert(process.env.APPDATA && !process.env.MUZO_DATA_DIR);
const chromium = join(process.env.APPDATA, "Muzo");
const profile = {
  data: join(process.env.APPDATA, "muzo"),
  leveldb: join(chromium, "Local Storage", "leveldb"),
};
const legacyMain = join(
  dirname(legacyExecutable),
  "resources",
  "app",
  "dist-electron",
  "main.cjs",
);
await assert.rejects(access(join(profile.data, "muzo-rust.sqlite")));
await mkdir(fixture);
const feed = await startLegacyUpdateFeed(candidate, manifest, legacyMain);
const legacy = spawn(legacyExecutable, ["--remote-debugging-port=0"], {
  stdio: "ignore",
});
let startupError;
legacy.once("error", (error) => {
  startupError = error;
});
try {
  const fixturePreferences = join(fixture, "preferences.json");
  const seeded = await execute(process.execPath, [
    fileURLToPath(new URL("./seed-upgrade-profile.mjs", import.meta.url)),
    chromium,
    join(fixture, "Music"),
    fixturePreferences,
  ]);
  process.stdout.write(seeded.stdout);
  assert.ifError(startupError);
  const preferences = JSON.parse(await readFile(fixturePreferences, "utf8"));
  const release = await fetch(new URL("release", feed.address)).then(
    (response) => response.json(),
  );
  const version = release.tag_name.slice(1);
  const windowsDriver = fileURLToPath(
    new URL("./windows-update-window.ps1", import.meta.url),
  );
  const driveWindow = async (journey) => {
    const result = await execute("powershell.exe", [
      "-NoProfile",
      "-NonInteractive",
      "-File",
      windowsDriver,
      "-Journey",
      journey,
      "-LegacyProcessId",
      String(legacy.pid),
      "-Executable",
      legacyExecutable,
    ]);
    process.stdout.write(result.stdout);
  };
  const socket = await connectToRenderer(chromium);
  const evaluate = renderer(socket);
  let expected;
  try {
    const check = await evaluate(
      "window.__MUZO_RUNTIME__.updates.command('check')",
    );
    assert.equal(check?.ok, true, JSON.stringify(check));
    await waitForUpdateButton(evaluate, "Update available v" + version);
    await evaluate(rendererButton + ".click()");
    await waitForUpdateButton(evaluate, "Restart to install v" + version);
    expected = captureLegacyProfile(profile, preferences);
    await evaluate(rendererButton + ".click()");
    await driveWindow("Confirm");
  } finally {
    socket.close();
  }

  await waitFor(
    async () => {
      assert.equal(legacy.exitCode, 0);
      await assert.rejects(access(legacyMain));
      verifyProfileUpgrade(profile, expected);
    },
    restartTimeoutMs,
    "The installed Rust app did not restart with the Electron profile",
  );
  await driveWindow("Reopened");
  console.log(
    "Verified Download and Restart in the published Electron Windows app.",
  );
  console.log(
    "The native installer replaced Electron and reopened its migrated profile.",
  );
} finally {
  if (legacy.exitCode === null) legacy.kill();
  await feed.close();
}
