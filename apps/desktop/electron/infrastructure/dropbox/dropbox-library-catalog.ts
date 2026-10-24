import type {
  DropboxLibraryCatalog,
  DropboxSourceFile,
} from "../../application/interfaces/dropbox-interfaces";
import type { DropboxEntry } from "./dropbox-api-client";

const audioExtensions = new Set(["mp3", "flac", "m4a", "ogg", "opus", "wav"]);

export interface DropboxFileLister {
  listFolderRecursive(
    accessToken: string,
    path: string,
  ): Promise<DropboxEntry[]>;
}

export class DropboxLibraryCatalogAdapter implements DropboxLibraryCatalog {
  constructor(private readonly dropboxFileLister: DropboxFileLister) {}

  async listAudioFiles(
    accessToken: string,
    root: string,
  ): Promise<DropboxSourceFile[]> {
    const entries = await this.dropboxFileLister.listFolderRecursive(
      accessToken,
      root,
    );

    return entries
      .filter((entry) => entry.tag === "file")
      .filter(isSupportedAudioFile)
      .map(sourceFileFromEntry);
  }
}

function isSupportedAudioFile(entry: DropboxEntry): boolean {
  const extension = entry.name.split(".").at(-1);
  return extension ? audioExtensions.has(extension.toLowerCase()) : false;
}

function sourceFileFromEntry(entry: DropboxEntry): DropboxSourceFile {
  return {
    path: entry.pathDisplay ?? entry.pathLower ?? entry.name,
    size: entry.size ?? 0,
    modifiedEpochSeconds: entry.serverModified
      ? Math.floor(Date.parse(entry.serverModified) / 1000)
      : 0,
  };
}
