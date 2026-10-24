import { copyFile, mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";
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

export async function registerAppImageMenuEntry(
  options: AppImageMenuEntryOptions,
): Promise<Result<"registered">> {
  try {
    const directory = join(options.dataDirectory, "applications");
    const icon = join(options.dataDirectory, "muzo/appimage.png");
    await mkdir(directory, { recursive: true });
    await mkdir(join(options.dataDirectory, "muzo"), { recursive: true });
    await copyFile(options.iconPath, icon);
    await writeFile(
      join(directory, "muzo.desktop"),
      [
        "[Desktop Entry]",
        "Type=Application",
        "Name=Muzo",
        "Comment=Muzo desktop music player",
        `Exec=${executable(options.appImagePath)}`,
        `Icon=${desktopString(icon)}`,
        "Terminal=false",
        "StartupWMClass=muzo",
        "Categories=AudioVideo;Audio;Player;",
        "",
      ].join("\n"),
      { mode: 0o644 },
    );
    return success("registered");
  } catch {
    return failure("storage", "Could not add Muzo to your application menu.");
  }
}
