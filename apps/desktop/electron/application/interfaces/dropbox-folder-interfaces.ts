export interface DropboxFolder {
  id: string;
  name: string;
  path: string;
}

export interface DropboxFolderListing {
  folders: DropboxFolder[];
  addedPaths: string[];
}

export interface DropboxFolderReader {
  listFolders(accessToken: string, path: string): Promise<DropboxFolder[]>;
}
