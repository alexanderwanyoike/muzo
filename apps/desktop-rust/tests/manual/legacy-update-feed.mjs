import assert from "node:assert/strict";
import { createReadStream } from "node:fs";
import { readFile, stat, writeFile } from "node:fs/promises";
import { createServer } from "node:http";
import { basename } from "node:path";

const legacyFeed =
  /var MUZO_UPDATE_FEED = \{\s*provider: "github",\s*owner: "alexanderwanyoike",\s*repo: "muzo"\s*\};/u;
const releaseEndpoint =
  "https://api.github.com/repos/alexanderwanyoike/muzo/releases/latest";
const releases = "https://github.com/alexanderwanyoike/muzo/releases";

function describeRelease(version) {
  const names = [
    "Muzo-" + version + "-windows-x64-setup.exe",
    "Muzo-" + version + "-linux-x64.AppImage",
    "Muzo-" + version + "-macos-arm64.dmg",
    "Muzo-" + version + "-macos-x64.dmg",
    "muzo-" + version + "-release.json",
    "SHA256SUMS",
    "latest.yml",
    "latest-linux.yml",
  ];
  return {
    tag_name: "v" + version,
    draft: false,
    prerelease: false,
    assets: names.map((name) => ({
      name,
      browser_download_url: releases + "/download/v" + version + "/" + name,
    })),
  };
}

function configureLegacyApp(source, address) {
  assert.match(source, legacyFeed, "The released Electron feed could not be located");
  const routed = source.replace(
    legacyFeed,
    'var MUZO_UPDATE_FEED = { provider: "generic", url: ' + JSON.stringify(address) + " };",
  );
  const requestTarget = JSON.stringify(new URL("release", address).href);
  const fixture = [
    "const fixtureFetch = globalThis.fetch;",
    "globalThis.fetch = (input, options) => fixtureFetch(input === " +
      JSON.stringify(releaseEndpoint) + " ? " + requestTarget + " : input, options);",
    'const fixtureDialog = require("electron").dialog;',
    "const originalDialog = fixtureDialog.showMessageBox.bind(fixtureDialog);",
    "fixtureDialog.showMessageBox = (options) => options.message?.startsWith(\"Restart Muzo to install\") " +
      "? Promise.resolve({ response: 1, checkboxChecked: false }) : originalDialog(options);",
  ];
  return fixture.join("\n") + "\n" + routed;
}

export async function startLegacyUpdateFeed(installer, manifest, legacyMain) {
  const name = basename(installer);
  const version = /^Muzo-(\d+\.\d+\.\d+)-(?:windows-x64-setup\.exe|linux-x64\.AppImage)$/u.exec(name)?.[1];
  assert(version, "The candidate must have a supported release filename");
  const bundle = await readFile(legacyMain, "utf8");
  const manifestName = basename(manifest);
  const manifestContents = await readFile(manifest);
  const { size } = await stat(installer);
  const release = describeRelease(version);
  const server = createServer((request, response) => {
    const path = new URL(request.url ?? "/", "http://127.0.0.1").pathname;
    if (path === "/release") {
      response.setHeader("Content-Type", "application/json");
      response.end(JSON.stringify(release));
    } else if (path === "/" + manifestName) {
      response.end(manifestContents);
    } else if (path === "/" + name) {
      response.setHeader("Content-Length", size);
      const stream = createReadStream(installer);
      stream.once("error", (error) => response.destroy(error));
      stream.pipe(response);
    } else {
      response.writeHead(404);
      response.end();
    }
  });
  const close = () =>
    new Promise((resolve) => {
      server.close(resolve);
      server.closeAllConnections();
    });
  try {
    await new Promise((resolve, reject) => {
      server.once("error", reject);
      server.listen(0, "127.0.0.1", resolve);
    });
    const listening = server.address();
    assert(listening && typeof listening !== "string");
    const address = "http://127.0.0.1:" + listening.port + "/";
    await writeFile(legacyMain, configureLegacyApp(bundle, address));
    return { address, close };
  } catch (error) {
    await close();
    throw error;
  }
}
