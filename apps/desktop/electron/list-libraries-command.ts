import type { LibraryDto } from "../src/types";
import type { CommandDependencies, CommandHandler } from "./command-handler";

export class ListLibrariesCommand implements CommandHandler {
  readonly command = "list_libraries";

  handle(
    _args: unknown,
    dependencies: CommandDependencies,
  ): Promise<LibraryDto[]> {
    return dependencies.libraries.list();
  }
}
