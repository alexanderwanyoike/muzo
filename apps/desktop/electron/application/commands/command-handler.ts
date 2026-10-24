import type { Result } from "../result";
export interface CommandHandler {
  readonly command: string;
  handle(args: unknown): Promise<Result<unknown>>;
}
