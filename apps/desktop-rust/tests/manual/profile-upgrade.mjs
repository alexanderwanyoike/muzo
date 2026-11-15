import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { DatabaseSync } from "node:sqlite";
import { setTimeout as delay } from "node:timers/promises";
import { pathToFileURL } from "node:url";

const legacyDatabase = "muzo.sqlite";
const nativeDatabase = "muzo-rust.sqlite";

const queries = {
  sources:
    "SELECT id, name, lower(kind) AS kind, location FROM libraries ORDER BY id",
  tracks: `SELECT id, library_id, title, artist, duration_seconds, file_path,
        album, track_number, disc_number, genre, year,
        override_title, override_artist, override_album, override_track_number,
        override_disc_number, override_genre, override_year FROM tracks ORDER BY id`,
  playlists: "SELECT id, name FROM playlists ORDER BY id",
  entries:
    "SELECT id, playlist_id, track_id, position FROM playlist_entries ORDER BY playlist_id, position",
  history:
    "SELECT id, library_id, track_id, played_at_unix_seconds FROM play_history ORDER BY id",
};

function readCatalogue(path) {
  const database = new DatabaseSync(path, { readOnly: true });
  try {
    return Object.fromEntries(
      Object.entries(queries).map(([table, query]) => [
        table,
        database.prepare(query).all(),
      ]),
    );
  } finally {
    database.close();
  }
}

function fingerprint({ data, leveldb }) {
  const sha256 = (path) =>
    createHash("sha256").update(readFileSync(path)).digest("hex");
  const settings = readdirSync(leveldb, { withFileTypes: true }).filter(
    (entry) => entry.isFile() && entry.name !== "LOCK",
  );
  return {
    catalogue: sha256(join(data, legacyDatabase)),
    chromium: Object.fromEntries(
      settings.map(({ name }) => [name, sha256(join(leveldb, name))]),
    ),
  };
}

export function captureLegacyProfile(profile, preferences) {
  return {
    catalogue: readCatalogue(join(profile.data, legacyDatabase)),
    preferences,
    legacy: fingerprint(profile),
  };
}

export function verifyProfileUpgrade(profile, baseline) {
  const catalogue = readCatalogue(join(profile.data, nativeDatabase));
  for (const table of Object.keys(queries)) {
    assert.deepStrictEqual(
      catalogue[table],
      baseline.catalogue[table],
      `Migrated catalogue changed ${table}`,
    );
  }

  const readJson = (name) =>
    JSON.parse(readFileSync(join(profile.data, name), "utf8"));
  assert.deepStrictEqual(
    readJson("appearance-rust.json"),
    baseline.preferences.appearance,
    "The appearance preferences changed",
  );
  assert.equal(
    readJson("sidebar-rust.json"),
    baseline.preferences.sidebar,
    "The sidebar width changed",
  );
  assert.deepStrictEqual(
    fingerprint(profile),
    baseline.legacy,
    "The legacy profile changed",
  );
}

const migrationTimeoutMs = 20_000;
const migrationPollMs = 250;

async function waitForUpgrade(profile, baseline) {
  const deadline = Date.now() + migrationTimeoutMs;
  let failure;
  do {
    try {
      verifyProfileUpgrade(profile, baseline);
      return;
    } catch (error) {
      failure = error;
      await delay(migrationPollMs);
    }
  } while (Date.now() < deadline);
  throw new Error("The installed app did not preserve the legacy profile", {
    cause: failure,
  });
}

if (
  process.argv[1] &&
  pathToFileURL(process.argv[1]).href === import.meta.url
) {
  const [action, data, leveldb, baselinePath, preferencesPath] =
    process.argv.slice(2);
  const profile = { data, leveldb };
  assert(
    data && leveldb && baselinePath,
    "Expected the data, Chromium storage, and baseline paths",
  );
  if (action === "capture") {
    assert(preferencesPath, "Expected the preferences recorded from Electron");
    const preferences = JSON.parse(readFileSync(preferencesPath, "utf8"));
    const baseline = captureLegacyProfile(profile, preferences);
    writeFileSync(baselinePath, JSON.stringify(baseline));
  } else {
    assert.equal(action, "verify");
    await waitForUpgrade(
      profile,
      JSON.parse(readFileSync(baselinePath, "utf8")),
    );
    console.log(
      "Preserved source and track IDs, metadata edits, ordered repeated playlist entries, play history, appearance, and sidebar width. Legacy files are unchanged.",
    );
  }
}
