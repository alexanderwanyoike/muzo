import {
  copyFile,
  link,
  mkdir,
  readFile,
  readdir,
  rename,
  rm,
  writeFile,
} from "node:fs/promises";
import { randomUUID } from "node:crypto";
import { isAbsolute, join } from "node:path";
import { failure, success, type Result } from "./application/result";

export interface AppImageMenuEntryOptions {
  appImagePath: string;
  dataDirectory: string;
  iconPath: string;
}

function desktopString(value: string): string {
  return value.replaceAll("\\", "\\\\");
}

function executable(value: string): string {
  const quoted = value.replace(/[\\"`$]/g, (character) => `\\${character}`);
  // Desktop string escapes are decoded before Exec argument quoting.
  return `"${desktopString(quoted).replaceAll("%", "%%")}"`;
}

function desktopEntryFields(entry: string): Map<string, string> {
  const fields = new Map<string, string>();
  let section = "";
  for (const raw of entry.split("\n")) {
    const line = raw.trim();
    if (line.startsWith("[")) {
      section = line;
      continue;
    }
    if (section !== "[Desktop Entry]" || line.startsWith("#")) continue;
    const separator = line.indexOf("=");
    if (separator > 0) fields.set(line.slice(0, separator).trim(), line.slice(separator + 1).trim());
  }
  return fields;
}

const managedKey = "X-Muzo-AppImage-Integration=true";
type AppImageMenuState =
  | { kind: "missing" | "managed"; filename: string }
  | { kind: "external" };

async function appImageMenuState(
  dataDirectory: string,
): Promise<Result<AppImageMenuState>> {
  try {
    const directory = join(dataDirectory, "applications");
    let files: string[];
    try {
      files = await readdir(directory);
    } catch (error) {
      if (error instanceof Error && "code" in error && error.code === "ENOENT")
        return success({ kind: "missing", filename: "muzo.desktop" });
      throw error;
    }
    let managed: string | undefined;
    for (const name of files.filter((file) => file.endsWith(".desktop"))) {
      let entry: string;
      try {
        entry = await readFile(join(directory, name), "utf8");
      } catch {
        if (name === "muzo.desktop" || name === "Muzo.cinnamon-generated.desktop")
          return failure("storage", "Could not read Muzo's existing menu shortcut.");
        continue;
      }
      const fields = desktopEntryFields(entry);
      const cinnamonTemporary =
        name === "Muzo.cinnamon-generated.desktop" &&
        fields.get("Name") === "Muzo" &&
        /^\/tmp\/\.mount_[^\n]*\/muzo(?:\s|$)/.test(fields.get("Exec") ?? "");
      if (fields.get("X-Muzo-AppImage-Integration") === "true" || cinnamonTemporary) {
        managed = name;
      } else if (
        name === "muzo.desktop" ||
        fields.get("StartupWMClass") === "muzo"
      ) {
        return success({ kind: "external" });
      }
    }
    return success({
      kind: managed ? "managed" : "missing",
      filename: managed ?? "muzo.desktop",
    });
  } catch {
    return failure("storage", "Could not read your application menu entries.");
  }
}

export async function registerAppImageMenuEntry(
  options: AppImageMenuEntryOptions,
): Promise<Result<"registered" | "existing">> {
  if (
    ![options.appImagePath, options.dataDirectory, options.iconPath].every(
      (path) =>
        isAbsolute(path) &&
        ![...path].some(
          (character) => character < " " || character === "\u007f",
        ),
    ) ||
    options.appImagePath.includes("=")
  ) {
    return failure(
      "invalidInput",
      "The AppImage or menu location cannot be used in a desktop launcher.",
    );
  }
  const state = await appImageMenuState(options.dataDirectory);
  if (!state.ok) return state;
  if (state.value.kind === "external") return success("existing");
  const temporary = join(
    options.dataDirectory,
    "applications",
    `.muzo-${randomUUID()}.tmp`,
  );
  try {
    const directory = join(options.dataDirectory, "applications");
    const icon = join(options.dataDirectory, "muzo/appimage.png");
    await mkdir(directory, { recursive: true });
    await mkdir(join(options.dataDirectory, "muzo"), { recursive: true });
    await copyFile(options.iconPath, icon);
    await writeFile(
      temporary,
      [
        "[Desktop Entry]",
        "Type=Application",
        "Name=Muzo",
        "Comment=Muzo desktop music player",
        // GLib checks executable existence before expanding literal percent escapes.
        `Exec=${options.appImagePath.includes("%") ? "env " : ""}${executable(options.appImagePath)}`,
        `Icon=${desktopString(icon)}`,
        "Terminal=false",
        "StartupWMClass=muzo",
        "Categories=AudioVideo;Audio;Player;",
        managedKey,
        "",
      ].join("\n"),
      { mode: 0o644 },
    );
    if (state.value.kind === "managed") {
      await rename(temporary, join(directory, state.value.filename));
    } else {
      try {
        await link(temporary, join(directory, state.value.filename));
      } catch (error) {
        if (
          error instanceof Error &&
          "code" in error &&
          error.code === "EEXIST"
        )
          return success("existing");
        throw error;
      }
    }
    return success("registered");
  } catch {
    return failure("storage", "Could not add Muzo to your application menu.");
  } finally {
    await rm(temporary, { force: true }).catch(() => undefined);
  }
}
