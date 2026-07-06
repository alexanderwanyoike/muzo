import { AddLibraryCommand } from "../application/commands/add-library-command";
import type {
  CommandDependencies,
  CommandHandler,
} from "../application/commands/command-handler";
import { ListLibrariesCommand } from "../application/commands/list-libraries-command";
import { ListTracksCommand } from "../application/commands/list-tracks-command";
import { createCommandDependencies } from "../composition/dependencies";

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
