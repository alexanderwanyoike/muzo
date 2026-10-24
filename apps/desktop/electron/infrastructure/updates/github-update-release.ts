import { RELEASE_TARGETS, isStableReleaseVersion, releaseAssetName } from "@muzo/distribution";
import { failure, success, type Result } from "../../application/result";

export const MUZO_RELEASES_URL = "https://github.com/alexanderwanyoike/muzo/releases";
const RELEASE_API_URL = "https://api.github.com/repos/alexanderwanyoike/muzo/releases/latest";
const UPDATE_REQUEST_TIMEOUT_MS = 15_000;
function record(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object";
}
export async function latestMuzoRelease(request: typeof fetch = fetch): Promise<Result<{ version: string }>> {
  try {
    const response = await request(RELEASE_API_URL, { headers: { Accept: "application/vnd.github+json" }, signal: AbortSignal.timeout(UPDATE_REQUEST_TIMEOUT_MS) });
    if (!response.ok) return failure("remote", "Could not check for updates. Try again when you are online.");
    const data: unknown = await response.json();
    if (!record(data) || typeof data.tag_name !== "string" || data.draft !== false || data.prerelease !== false || !Array.isArray(data.assets))
      return failure("remote", "The update release metadata is invalid.");
    const version = data.tag_name.slice(1);
    if (data.tag_name !== `v${version}` || !isStableReleaseVersion(version))
      return failure("remote", "The update must be a published stable release.");
    const assets: unknown[] = data.assets;
    const complete = RELEASE_TARGETS.every(({ id }) => {
      const name = releaseAssetName(id, version);
      return assets.filter((asset) => record(asset) && asset.name === name && asset.browser_download_url === `${MUZO_RELEASES_URL}/download/v${version}/${name}`).length === 1;
    });
    return complete ? success({ version }) : failure("remote", "The update release is missing a supported installer.");
  } catch {
    return failure("remote", "Could not check for updates. Try again when you are online.");
  }
}
