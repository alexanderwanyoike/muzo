import { readdir, stat } from "node:fs/promises";
import { join } from "node:path";

import type {
  AudioFileWalker,
  WalkedAudioFile,
} from "../../application/interfaces/scan-interfaces";

const audioExtensions = new Set(["mp3", "flac", "m4a", "ogg", "opus", "wav"]);

export class NodeAudioFileWalker implements AudioFileWalker {
  async walkAudioFiles(root: string): Promise<WalkedAudioFile[]> {
    const files: WalkedAudioFile[] = [];
    await walk(root, files);
    return files;
  }
}

async function walk(directory: string, files: WalkedAudioFile[]): Promise<void> {
  const entries = await readdir(directory, { withFileTypes: true });
  for (const entry of entries) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) {
      await walk(path, files);
      continue;
    }
    if (!entry.isFile() || !isAudioFile(entry.name)) {
      continue;
    }

    const metadata = await stat(path);
    files.push({
      path,
      size: metadata.size,
      mtime: Math.floor(metadata.mtimeMs / 1000),
    });
  }
}

function isAudioFile(path: string): boolean {
  const extension = path.split(".").at(-1);
  return extension ? audioExtensions.has(extension.toLowerCase()) : false;
}
