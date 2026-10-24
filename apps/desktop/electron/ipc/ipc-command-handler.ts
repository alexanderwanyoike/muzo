export interface IpcCommandHandler {
  readonly command: string;
  handle(args: unknown): Promise<unknown>;
}
