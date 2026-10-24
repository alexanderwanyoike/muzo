import { failure, type Result } from "../shared/result";
import { invokeCommand } from "./runtime-bridge";

export async function readTrackArtwork(
  libraryId: string,
  trackId: string,
): Promise<Result<string | null>> {
  try {
    const result: unknown = await invokeCommand("get_track_artwork", {
      input: { libraryId, trackId },
    });
    if (result && typeof result === "object" && "ok" in result) {
      if (
        result.ok === true &&
        "value" in result &&
        (result.value === null ||
          (typeof result.value === "string" &&
            result.value.startsWith("data:image/png;base64,")))
      )
        return { ok: true, value: result.value };
      if (
        result.ok === false &&
        "error" in result &&
        result.error &&
        typeof result.error === "object" &&
        "message" in result.error &&
        typeof result.error.message === "string"
      )
        return failure("remote", result.error.message);
    }
  } catch {
    return failure("remote", "Album artwork is temporarily unavailable.");
  }
  return failure("remote", "Album artwork is unavailable.");
}
