import { invoke } from "@tauri-apps/api/core";
import type {
  AddLibraryInputDto,
  LibraryDto,
  LibraryKindDto,
} from "./types";

export async function addLibrary(input: {
  name: string;
  kind: LibraryKindDto;
  location: string;
}): Promise<LibraryDto> {
  const dto: AddLibraryInputDto = {
    name: input.name,
    kind: input.kind,
    location: input.location,
  };
  return invoke<LibraryDto>("add_library", { input: dto });
}

export async function listLibraries(): Promise<LibraryDto[]> {
  return invoke<LibraryDto[]>("list_libraries");
}
