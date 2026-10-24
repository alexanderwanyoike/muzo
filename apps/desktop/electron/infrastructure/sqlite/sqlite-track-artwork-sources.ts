import { readFileSync } from "node:fs";
import initSqlJs from "sql.js";
import type {
  TrackArtworkSource,
  TrackArtworkSources,
} from "../../application/interfaces/track-artwork";
import { locateSqlWasm } from "./sql-wasm-path";

const sqlModule = initSqlJs({ locateFile: locateSqlWasm });
export class SqliteTrackArtworkSources implements TrackArtworkSources {
  constructor(private readonly dbPath: string) {}

  async find(
    libraryId: string,
    trackId: string,
  ): Promise<TrackArtworkSource | null> {
    const SQL = await sqlModule;
    const database = new SQL.Database(readFileSync(this.dbPath));
    try {
      const result = database.exec(
        `SELECT t.file_path, t.file_size, t.file_mtime, l.kind FROM tracks t JOIN libraries l ON l.id = t.library_id WHERE t.library_id = ? AND t.id = ?`,
        [libraryId, trackId],
      );
      const row = result[0]?.values[0];
      if (!row) return null;
      const [path, size, mtime, kind] = row;
      if (
        typeof path !== "string" ||
        typeof size !== "number" ||
        typeof mtime !== "number" ||
        (kind !== "Filesystem" && kind !== "Dropbox")
      )
        throw new Error("Invalid artwork source record");
      return {
        libraryId,
        trackId,
        path,
        revision: `${size}:${mtime}`,
        kind: kind === "Filesystem" ? "filesystem" : "dropbox",
      };
    } finally {
      database.close();
    }
  }
}
