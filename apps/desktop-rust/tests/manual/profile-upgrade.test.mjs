import assert from "node:assert/strict";
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { DatabaseSync } from "node:sqlite";
import { test } from "node:test";
import {
  captureLegacyProfile,
  verifyProfileUpgrade,
} from "./profile-upgrade.mjs";

const preferences = {
  appearance: { theme: "walnut", mode: "dark", accent: "glow-cyan" },
  sidebar: 312,
};

function fixture(t) {
  const root = mkdtempSync(join(tmpdir(), "muzo-profile-verification-"));
  t.after(() => rmSync(root, { recursive: true }));
  const data = join(root, "data");
  const leveldb = join(root, "Local Storage", "leveldb");
  mkdirSync(data);
  mkdirSync(leveldb, { recursive: true });
  writeFileSync(join(leveldb, "CURRENT"), "MANIFEST-000001\n");
  writeFileSync(join(leveldb, "000004.log"), "Chromium storage fixture");

  const legacy = join(data, "muzo.sqlite");
  const candidate = join(data, "muzo-rust.sqlite");
  const migrations = new URL("../../migrations/", import.meta.url);
  const database = new DatabaseSync(legacy);
  try {
    for (const name of readdirSync(migrations).sort()) {
      database.exec(readFileSync(new URL(name, migrations), "utf8"));
    }
    database.exec(`
      INSERT INTO libraries VALUES ('source', 'Rehearsals', 'filesystem', '/music');
      INSERT INTO tracks (id, library_id, title, artist, duration_seconds, file_path,
        file_size, file_mtime, override_title)
      VALUES ('track-a', 'source', 'First', 'Mara', 12, '/music/first.wav', 44, 1, 'First edit'),
        ('track-b', 'source', 'Second', 'Mara', 18, '/music/second.wav', 44, 1, NULL);
      INSERT INTO playlists VALUES ('playlist', 'Evening Walk');
      INSERT INTO playlist_entries VALUES ('entry-a', 'playlist', 'track-a', 0),
        ('entry-b', 'playlist', 'track-b', 1), ('entry-c', 'playlist', 'track-a', 2);
      INSERT INTO play_history VALUES ('play-a', 'source', 'track-a', 20),
        ('play-b', 'source', 'track-a', 21), ('play-c', 'source', 'track-b', 22);
    `);
  } finally {
    database.close();
  }
  copyFileSync(legacy, candidate);
  writeFileSync(
    join(data, "appearance-rust.json"),
    JSON.stringify(preferences.appearance),
  );
  writeFileSync(
    join(data, "sidebar-rust.json"),
    JSON.stringify(preferences.sidebar),
  );
  const profile = { data, leveldb };
  return {
    profile,
    candidate,
    baseline: JSON.parse(
      JSON.stringify(captureLegacyProfile(profile, preferences)),
    ),
  };
}

test("checks that the installed app preserved the catalogue and Chromium preferences", (t) => {
  const { profile, baseline } = fixture(t);
  assert.doesNotThrow(() => verifyProfileUpgrade(profile, baseline));
});

test("rejects changed catalogue records even when all row counts match", (t) => {
  const { profile, baseline, candidate } = fixture(t);
  const database = new DatabaseSync(candidate);
  try {
    database.exec(
      "UPDATE playlist_entries SET track_id = 'track-b' WHERE id = 'entry-c'",
    );
  } finally {
    database.close();
  }
  assert.throws(
    () => verifyProfileUpgrade(profile, baseline),
    /catalogue.*entries/i,
  );
});

test("rejects changed preferences and overwritten Chromium storage", (t) => {
  const { profile, baseline } = fixture(t);
  const sidebar = join(profile.data, "sidebar-rust.json");
  writeFileSync(sidebar, "240");
  assert.throws(() => verifyProfileUpgrade(profile, baseline), /sidebar/i);
  writeFileSync(sidebar, JSON.stringify(preferences.sidebar));
  writeFileSync(join(profile.leveldb, "000004.log"), "changed");
  assert.throws(
    () => verifyProfileUpgrade(profile, baseline),
    /legacy.*changed/i,
  );
});
