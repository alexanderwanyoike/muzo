export interface DropboxSourceFile {
  path: string;
  size: number;
  modifiedEpochSeconds: number;
}

export interface DropboxLibraryCatalog {
  listAudioFiles(
    accessToken: string,
    root: string,
  ): Promise<DropboxSourceFile[]>;
}
