import { homedir } from "node:os";
import { join } from "node:path";

export function muzoDatabasePath(env: NodeJS.ProcessEnv = process.env): string {
  return join(resolveMuzoDataDir(process.platform, env), "muzo.sqlite");
}

export function resolveMuzoDataDir(
  platform: NodeJS.Platform,
  env: NodeJS.ProcessEnv,
  home = homedir(),
): string {
  if (env.MUZO_DATA_DIR) {
    return env.MUZO_DATA_DIR;
  }

  if (platform === "win32") {
    return join(env.APPDATA ?? home, "muzo");
  }

  if (platform === "darwin") {
    return join(home, "Library", "Application Support", "muzo");
  }

  return join(env.XDG_DATA_HOME ?? join(home, ".local", "share"), "muzo");
}
