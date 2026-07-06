import { AddLibraryCommand } from "./add-library-command";
import type { CommandDependencies, CommandHandler } from "./command-handler";
import { createCommandDependencies } from "./dependencies";
import { ListLibrariesCommand } from "./list-libraries-command";
import { ListTracksCommand } from "./list-tracks-command";

const commandHandlers = new Map<string, CommandHandler>(
  [
    new AddLibraryCommand(),
    new ListLibrariesCommand(),
    new ListTracksCommand(),
  ].map((handler) => [handler.command, handler]),
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
