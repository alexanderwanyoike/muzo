import type { LibraryRecord } from "./library-record";

export interface DropboxLibrarySettings {
  register(library: LibraryRecord, accountId: string): Promise<void>;
  accountFor(libraryId: string): Promise<string | null>;
  intervalMinutes(): Promise<number>;
  setIntervalMinutes(minutes: number): Promise<void>;
}
