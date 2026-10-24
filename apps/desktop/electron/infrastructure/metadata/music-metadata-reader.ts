import { basename, extname } from "node:path";

import { parseFile } from "music-metadata";

import type {
  AudioMetadata,
  AudioMetadataReader,
} from "../../application/interfaces/scan-interfaces";

export class MusicMetadataReader implements AudioMetadataReader {
  async read(path: string): Promise<AudioMetadata> {
    const metadata = await parseFile(path);
    const title = metadata.common.title ?? filenameWithoutExtension(path);
    const artist = metadata.common.artist ?? "Unknown Artist";

    return {
      title,
      artist,
      album: metadata.common.album ?? null,
      trackNumber: metadata.common.track?.no ?? null,
      discNumber: metadata.common.disk?.no ?? null,
      genre: metadata.common.genre?.[0] ?? null,
      year: metadata.common.year ?? null,
      durationSeconds: Math.floor(metadata.format.duration ?? 0),
    };
  }
}

function filenameWithoutExtension(path: string): string {
  const file = basename(path);
  const extension = extname(file);
  return extension ? file.slice(0, -extension.length) : file;
}
