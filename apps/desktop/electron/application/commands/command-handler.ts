export interface CommandHandler {
  readonly command: string;
  handle(args: unknown): Promise<unknown>;
}
