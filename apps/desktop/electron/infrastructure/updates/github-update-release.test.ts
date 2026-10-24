// @vitest-environment node
import { expect, it, vi } from "vitest";
import { latestMuzoRelease } from "./github-update-release";

function release() {
  return { tag_name: "v0.0.3", draft: false, prerelease: false, assets: ["linux-x64.AppImage", "windows-x64-setup.exe", "macos-arm64.dmg", "macos-x64.dmg"].map((suffix) => ({ name: `Muzo-0.0.3-${suffix}`, browser_download_url: `https://github.com/alexanderwanyoike/muzo/releases/download/v0.0.3/Muzo-0.0.3-${suffix}` })) };
}
it("reads only the fixed public repository's complete published stable release", async () => {
  const request = vi.fn<typeof fetch>(async () => new Response(JSON.stringify(release())));
  expect(await latestMuzoRelease(request)).toEqual({ ok: true, value: { version: "0.0.3" } });
  expect(request.mock.calls[0]?.[0]).toBe("https://api.github.com/repos/alexanderwanyoike/muzo/releases/latest");
});
it.each(["draft", "prerelease", "foreign", "partial", "unstable", "network"])("rejects an unusable update release: %s", async (kind) => {
  const data = release();
  if (kind === "draft") data.draft = true;
  if (kind === "prerelease") data.prerelease = true;
  if (kind === "foreign") data.assets[0].browser_download_url = "https://example.com/update.AppImage";
  if (kind === "partial") data.assets.pop();
  if (kind === "unstable") data.tag_name = "v0.0.3-beta.1";
  expect((await latestMuzoRelease(async () => new Response(JSON.stringify(data), { status: kind === "network" ? 503 : 200 }))).ok).toBe(false);
});
