import type { Result } from "../application/result";
import type { IpcCommandHandler } from "./ipc-command-handler";

export class LegacyResultCommandAdapter implements IpcCommandHandler {
  constructor(private readonly handler: { readonly command: string; handle(args: unknown): Promise<Result<unknown>> }) {}

  get command(): string { return this.handler.command; }

  async handle(args: unknown): Promise<unknown> {
    const result = await this.handler.handle(args);
    // The existing renderer bridge expects rejection; keep that conversion at IPC.
    if (!result.ok) throw result.error;
    return result.value;
  }
}
