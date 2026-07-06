import type { AddLibraryInputDto, LibraryDto } from "../../../src/types";
import type { CommandDependencies, CommandHandler } from "./command-handler";

export class AddLibraryCommand implements CommandHandler {
  readonly command = "add_library";

  async handle(
    args: unknown,
    dependencies: CommandDependencies,
  ): Promise<LibraryDto> {
    const input = parseAddLibraryArgs(args);

    if (input.name.trim().length === 0) {
      throw { kind: "emptyName" };
    }

    if (input.location.trim().length === 0) {
      throw { kind: "emptyLocation" };
    }

    const library: LibraryDto = {
      id: dependencies.generateLibraryId(),
      name: input.name,
      kind: input.kind,
      location: input.location,
    };
    await dependencies.libraries.add(library);
    return library;
  }
}

function parseAddLibraryArgs(args: unknown): AddLibraryInputDto {
  if (!args || typeof args !== "object" || !("input" in args)) {
    throw { kind: "repository", message: "missing add_library input" };
  }
  const input = (args as { input: unknown }).input;
  if (!input || typeof input !== "object") {
    throw { kind: "repository", message: "invalid add_library input" };
  }
  return input as AddLibraryInputDto;
}
