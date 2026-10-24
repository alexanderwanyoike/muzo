import { failure, type Result } from "../../../shared/result";
import type { DropboxAccountService } from "../dropbox-account-service";
import type { CommandHandler } from "./command-handler";

export type DropboxAccountCommandName =
  | "dropbox_begin" | "dropbox_complete" | "dropbox_status" | "dropbox_disconnect";

export class DropboxAccountCommand implements CommandHandler {
  constructor(
    readonly command: DropboxAccountCommandName,
    private readonly service: DropboxAccountService,
  ) {}

  async handle(args: unknown): Promise<Result<unknown>> {
    if (this.command === "dropbox_status") return this.service.status();
    if (this.command === "dropbox_disconnect") return this.service.disconnect();
    const field = this.command === "dropbox_begin" ? "clientId" : "code";
    const input = args && typeof args === "object" && "input" in args ? args.input : null;
    if (!input || typeof input !== "object" || !(field in input)) {
      return failure("invalidInput", `Missing Dropbox ${field}.`);
    }
    const value: unknown = Reflect.get(input, field);
    if (typeof value !== "string" || !value.trim()) {
      return failure("invalidInput", `Enter a valid Dropbox ${field}.`);
    }
    return this.command === "dropbox_begin"
      ? this.service.begin(value.trim()) : this.service.complete(value.trim());
  }
}
