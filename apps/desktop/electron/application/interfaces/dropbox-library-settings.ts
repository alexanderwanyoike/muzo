export interface DropboxLibrarySettings {
  bind(libraryId: string, accountId: string): Promise<void>;
  accountFor(libraryId: string): Promise<string | null>;
  intervalMinutes(): Promise<number>;
  setIntervalMinutes(minutes: number): Promise<void>;
}
