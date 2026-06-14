export type LibraryKindDto = "filesystem" | "dropbox";

export interface LibraryDto {
  id: string;
  name: string;
  kind: LibraryKindDto;
  location: string;
}

export interface AddLibraryInputDto {
  name: string;
  kind: LibraryKindDto;
  location: string;
}

export type AddLibraryErrorDto =
  | { kind: "emptyName" }
  | { kind: "emptyLocation" }
  | { kind: "repository"; message: string };

export function errorMessage(err: AddLibraryErrorDto): string {
  switch (err.kind) {
    case "emptyName":
      return "Library name must not be empty.";
    case "emptyLocation":
      return "Library location must not be empty.";
    case "repository":
      return `Could not save the library: ${err.message}`;
  }
}
