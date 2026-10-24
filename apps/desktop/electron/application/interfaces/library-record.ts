export type LibraryKind = "filesystem" | "dropbox";

export interface LibraryRecord {
  id: string;
  name: string;
  kind: LibraryKind;
  location: string;
}

export interface AddLibraryInput {
  name: string;
  kind: LibraryKind;
  location: string;
}

