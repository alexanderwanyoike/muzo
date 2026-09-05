import type { CommandHandler } from "../application/commands/command-handler";

export class CommandDispatcher {
  private readonly commandHandlers: Map<string, CommandHandler>;

  constructor(commandHandlers: CommandHandler[]) {
    this.commandHandlers = new Map(
      commandHandlers.map((handler) => [handler.command, handler]),
    );
  }

  async handleElectronCommand(command: string, args?: unknown): Promise<unknown> {
    const handler = this.commandHandlers.get(command);
    if (!handler) {
      throw new Error(`Electron command is not implemented: ${command}`);
    }

    return handler.handle(args);
  }
}
