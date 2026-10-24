import { failure, type Result } from "../shared/result";

export default async function playlistRequest<T>(
  operation: () => Promise<T>,
  fallback: string,
): Promise<Result<T>> {
  try {
    return { ok: true, value: await operation() };
  } catch (error) {
    const message =
      typeof error === "object" &&
      error !== null &&
      "message" in error &&
      typeof error.message === "string"
        ? error.message
        : fallback;
    return failure("repository", message);
  }
}
