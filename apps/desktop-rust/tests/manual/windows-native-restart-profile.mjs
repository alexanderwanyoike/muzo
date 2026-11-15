import assert from "node:assert/strict";
import { DatabaseSync } from "node:sqlite";

const [action, catalogue] = process.argv.slice(2);
assert(catalogue && ["seed", "verify"].includes(action));

const playlist = { id: "rust-restart-fixture", name: "Rust-only playlist" };
const database = new DatabaseSync(catalogue, { readOnly: action === "verify" });
try {
  if (action === "seed") {
    database
      .prepare("INSERT INTO playlists (id, name) VALUES (?, ?)")
      .run(playlist.id, playlist.name);
  } else {
    const saved = database
      .prepare("SELECT name FROM playlists WHERE id = ?")
      .get(playlist.id);
    assert.equal(saved?.name, playlist.name, "The Rust-only playlist is missing");
  }
} finally {
  database.close();
}
