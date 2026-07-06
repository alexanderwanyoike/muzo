import { AddLibraryCommand } from "./add-library-command";
import type { CommandDependencies, CommandHandler } from "./command-handler";
import { createCommandDependencies } from "./dependencies";
import { ListLibrariesCommand } from "./list-libraries-command";

const commandHandlers = new Map<string, CommandHandler>(
  [new AddLibraryCommand(), new ListLibrariesCommand()].map((handler) => [
    handler.command,
    handler,
  ]),
);

export async function handleElectronCommand(
  command: string,
  args?: unknown,
  dependencies: CommandDependencies = createCommandDependencies(),
): Promise<unknown> {
  const handler = commandHandlers.get(command);
  if (!handler) {
    throw new Error(`Electron command is not implemented: ${command}`);
  }

  return handler.handle(args, dependencies);
}
