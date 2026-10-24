export interface ApplicationFailure {
  kind:
    | "emptyName"
    | "emptyLocation"
    | "libraryNotFound"
    | "notDropboxLibrary"
    | "playlistNotFound"
    | "invalidEntryOrder"
    | "trackNotFound"
    | "unsupportedFileType"
    | "repository"
    | "invalidInput"
    | "notConnected"
    | "authorization"
    | "storage"
    | "remote";
  message: string;
  libraryId?: string;
}

export type Result<T> =
  | { ok: true; value: T }
  | { ok: false; error: ApplicationFailure };

export function success<T>(value: T): Result<T> {
  return { ok: true, value };
}
export function failure(
  kind: ApplicationFailure["kind"],
  message: string,
  details: { libraryId?: string } = {},
): Result<never> {
  return { ok: false, error: { kind, message, ...details } };
}
export async function repositoryResult<T>(
  operation: () => Promise<T>,
): Promise<Result<T>> {
  try {
    return success(await operation());
  } catch {
    return failure("repository", "Could not complete the storage operation.");
  }
}
