import type { LibraryScanner, ScanReport } from "../scan-library-service";
import type { Result } from "../result";
import type { CommandHandler } from "./command-handler";
import { parseInput, parseNonEmptyString } from "./command-arguments";

export class ScanLibraryCommand implements CommandHandler {
  readonly command = "scan_library";

  constructor(private readonly service: LibraryScanner) {}

  async handle(args: unknown): Promise<Result<ScanReport>> {
    const input = parseInput(args, this.command);
    if (!input.ok) return input;
    const libraryId = parseNonEmptyString(
      input.value,
      "libraryId",
      this.command,
    );
    if (!libraryId.ok) return libraryId;
    return this.service.scan(libraryId.value);
  }
}
