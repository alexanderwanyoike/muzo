import assert from "node:assert/strict";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { startLegacyUpdateFeed } from "./legacy-update-feed.mjs";

test("the installed updater receives the candidate manifest and its exact bytes", async () => {
  const fixture = await mkdtemp(join(tmpdir(), "muzo-native-update-feed-"));
  const executable = "Muzo-0.0.6-windows-x64-setup.exe";
  const installer = join(fixture, executable);
  const metadata = join(fixture, "latest.yml");
  const legacy = join(fixture, "main.cjs");
  const payload = "candidate native installer";
  const manifest = "version: 0.0.6";
  let feed;
  try {
    await writeFile(installer, payload);
    await writeFile(metadata, manifest);
    await writeFile(
      legacy,
      'var MUZO_UPDATE_FEED = {\n  provider: "github",\n  owner: "alexanderwanyoike",\n  repo: "muzo"\n};\n',
    );
    feed = await startLegacyUpdateFeed(installer, metadata, legacy);
    const releaseResponse = await fetch(new URL("release", feed.address));
    const release = await releaseResponse.json();
    assert.equal(release.tag_name, "v0.0.6");
    assert(release.assets.some(({ name }) => name === executable));
    assert(
      release.assets.some(({ name }) => name === "muzo-0.0.6-release.json"),
    );
    assert.equal(
      await fetch(new URL("latest.yml?cache=1", feed.address)).then((res) =>
        res.text(),
      ),
      manifest,
    );
    assert.equal(
      await fetch(new URL(executable, feed.address)).then((res) => res.text()),
      payload,
    );
    assert.match(await readFile(legacy, "utf8"), /provider: "generic"/u);
    assert.equal((await fetch(new URL("wrong.exe", feed.address))).status, 404);
  } finally {
    await feed?.close();
    await rm(fixture, { recursive: true, force: true });
  }
});
