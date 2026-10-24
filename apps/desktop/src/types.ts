import type { AddLibraryInput, LibraryKind, LibraryRecord } from "../electron/application/interfaces/library-record";

export type LibraryKindDto = LibraryKind;
export type LibraryDto = LibraryRecord;
export type AddLibraryInputDto = AddLibraryInput;

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
