import { execFile } from "node:child_process";
import { createHash } from "node:crypto";
import { createReadStream } from "node:fs";
import { cp, readFile, stat, writeFile } from "node:fs/promises";
import { createServer } from "node:http";
import { createRequire } from "node:module";
import { join } from "node:path";
import { promisify } from "node:util";
import { build } from "esbuild";
import {
  RELEASE_TARGETS,
  releaseAssetName,
  releaseManifestName,
} from "@muzo/distribution";

export async function previousAppImage(
  root: string,
  desktop: string,
  payload: string,
) {
  const metadata = JSON.parse(
    await readFile(join(desktop, "package.json"), "utf8"),
  ) as { version: string };
  const { version } = metadata;
  const bytes = await readFile(payload);
  const sha512 = createHash("sha512").update(bytes).digest("base64");
  const name = releaseAssetName("linux-x64", version);
  const base = `https://github.com/alexanderwanyoike/muzo/releases/download/v${version}`;
  const server = createServer((request, response) => {
    if (request.url === "/release") {
      response.setHeader("Content-Type", "application/json");
      response.end(
        JSON.stringify({
          tag_name: `v${version}`,
          draft: false,
          prerelease: false,
          assets: [
            ...RELEASE_TARGETS.map(({ id }) => releaseAssetName(id, version)),
            releaseManifestName(version),
            "SHA256SUMS",
            "latest.yml",
            "latest-linux.yml",
          ].map((asset) => ({
            name: asset,
            browser_download_url: `${base}/${asset}`,
          })),
        }),
      );
    } else if (request.url?.startsWith("/latest-linux.yml")) {
      response.end(
        JSON.stringify({
          version,
          path: name,
          sha512,
          files: [{ url: name, sha512, size: bytes.length }],
        }),
      );
    } else if (request.url === `/${name}`) {
      response.writeHead(200, { "Content-Length": bytes.length });
      createReadStream(payload).pipe(response);
    } else {
      response.writeHead(404);
      response.end();
    }
  });
  await new Promise<void>((resolve) => {
    server.listen(0, "127.0.0.1", resolve);
  });
  const address = server.address();
  if (!address || typeof address === "string")
    throw new Error("Update feed unavailable");
  const url = `http://127.0.0.1:${address.port}`;
  const packaged = join(root, "previous");
  try {
    await cp(join(desktop, "out/Muzo-linux-x64"), packaged, {
      recursive: true,
    });
    const appDirectory = join(packaged, "resources/app");
    const packageFile = join(appDirectory, "package.json");
    const previous = JSON.parse(await readFile(packageFile, "utf8")) as Record<
      string,
      unknown
    >;
    await writeFile(
      packageFile,
      JSON.stringify({ ...previous, version: "0.0.1" }),
    );
    // Only the network boundary and confirmation are replaced; the installed updater and application stay real.
    await build({
      entryPoints: [join(desktop, "electron/main.ts")],
      outfile: join(appDirectory, "dist-electron/main.cjs"),
      bundle: true,
      platform: "node",
      format: "cjs",
      target: "node20",
      external: ["electron"],
      define: { "import.meta.url": "importMetaUrl" },
      banner: {
        js: `const importMetaUrl = require('node:url').pathToFileURL(__filename).href;
const request = globalThis.fetch; globalThis.fetch = (input, options) => request(input === 'https://api.github.com/repos/alexanderwanyoike/muzo/releases/latest' ? '${url}/release' : input, options);
require('electron').dialog.showMessageBox = async () => ({response: 1, checkboxChecked: false});`,
      },
      plugins: [
        {
          name: "local-update-feed",
          setup(builder) {
            builder.onLoad({ filter: /shared[/\\]update-feed\.ts$/ }, () => ({
              contents: `export const MUZO_UPDATE_FEED = {provider: 'generic', url: '${url}'};`,
              loader: "ts",
            }));
          },
        },
      ],
    });
    const config = JSON.parse(
      await readFile(join(desktop, "electron-builder.config.json"), "utf8"),
    ) as Record<string, unknown>;
    const configFile = join(root, "builder.json");
    await writeFile(
      configFile,
      JSON.stringify({
        ...config,
        directories: {
          output: join(root, "images"),
          buildResources: join(desktop, "assets/icons"),
        },
        linux: {
          target: ["AppImage"],
          executableName: "muzo",
          icon: join(desktop, "assets/icons/muzo.png"),
          category: "AudioVideo",
          artifactName: "Previous.AppImage",
        },
        publish: { provider: "generic", url },
      }),
    );
    await promisify(execFile)(
      process.execPath,
      [
        createRequire(import.meta.url).resolve("electron-builder/cli.js"),
        "--prepackaged",
        packaged,
        "--linux",
        "--x64",
        "--config",
        configFile,
        "--publish",
        "never",
      ],
      { cwd: desktop, timeout: 120_000, maxBuffer: 4 * 1024 * 1024 },
    );
    const image = join(root, "images/Previous.AppImage");
    await stat(image);
    return {
      image,
      version,
      close: () => {
        server.closeAllConnections();
        server.close();
      },
    };
  } catch (error) {
    server.closeAllConnections();
    server.close();
    throw error;
  }
}
