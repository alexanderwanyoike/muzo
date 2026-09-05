import type { CommandHandler } from "./command-handler";
import type {
  LibraryScanner,
  ScanReport,
} from "../scan-library-service";

export class ScanLibraryCommand implements CommandHandler {
  readonly command = "scan_library";

  constructor(private readonly scanLibraryService: LibraryScanner) {}

  handle(args: unknown): Promise<ScanReport> {
    const { libraryId } = parseScanLibraryArgs(args);
    return this.scanLibraryService.scan(libraryId);
  }
}

function parseScanLibraryArgs(args: unknown): { libraryId: string } {
  if (!args || typeof args !== "object" || !("input" in args)) {
    throw { kind: "repository", message: "missing scan_library input" };
  }

  const input = (args as { input: unknown }).input;
  if (!input || typeof input !== "object" || !("libraryId" in input)) {
    throw { kind: "repository", message: "invalid scan_library input" };
  }

  const libraryId = (input as { libraryId: unknown }).libraryId;
  if (typeof libraryId !== "string" || libraryId.trim().length === 0) {
    throw { kind: "repository", message: "invalid scan_library libraryId" };
  }

  return { libraryId };
}
