// @vitest-environment node
import { execFile } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";
import { build } from "esbuild";
import { expect, it } from "vitest";

it("renders bounded raster thumbnails with the real Electron image decoder", async () => {
  const directory = mkdtempSync(join(tmpdir(), "muzo-native-artwork-"));
  try {
    const bundle = join(directory, "thumbnail.cjs");
    await build({
      entryPoints: [
        fileURLToPath(
          new URL("./electron-artwork-thumbnails.ts", import.meta.url),
        ),
      ],
      outfile: bundle,
      bundle: true,
      platform: "node",
      format: "cjs",
      external: ["electron"],
    });
    const resultFile = join(directory, "result.json");
    const png = fileURLToPath(new URL("./fixtures/cover.png", import.meta.url));
    const script = join(directory, "verify.cjs");
    writeFileSync(
      script,
      `const {app,nativeImage}=require('electron');
const fs=require('node:fs');
const {ElectronArtworkThumbnails}=require(${JSON.stringify(bundle)});
app.whenReady().then(()=>{const thumbnails=new ElectronArtworkThumbnails(nativeImage);
const data=fs.readFileSync(${JSON.stringify(png)});
const result=thumbnails.create({format:'image/png',data});
const image=nativeImage.createFromDataURL(result);
const pixels=Buffer.alloc(256*256*4);
let seed=123;
for(let i=0;
i<pixels.length;
i++){seed^=seed<<13;
seed^=seed>>>17;
seed^=seed<<5;
pixels[i]=seed&255;
}const noise=thumbnails.create({format:'image/png',data:nativeImage.createFromBitmap(pixels,{width:256,height:256}).toPNG()});
fs.writeFileSync(${JSON.stringify(resultFile)},JSON.stringify({size:image.getSize(),uri:result,noiseSize:noise?nativeImage.createFromDataURL(noise).getSize():null,noiseLength:noise?.length??0,invalid:thumbnails.create({format:'image/png',data:Buffer.from('invalid')}),svg:thumbnails.create({format:'image/svg+xml',data:Buffer.from('<svg/>')})}));
app.quit();
});
`,
    );
    const require = createRequire(import.meta.url);
    const electronPath: unknown = require("electron");
    if (typeof electronPath !== "string")
      throw new Error("Electron executable unavailable");
    await promisify(execFile)(
      "xvfb-run",
      ["-a", electronPath, "--no-sandbox", script],
      {
        timeout: 30_000,
        env: {
          ...process.env,
          ELECTRON_RUN_AS_NODE: undefined,
          NODE_OPTIONS: undefined,
        },
      },
    );
    const result: unknown = JSON.parse(readFileSync(resultFile, "utf8"));
    if (result && typeof result === "object" && "noiseLength" in result) expect(result.noiseLength).toBeLessThanOrEqual(256 * 1024);
    expect(result).toMatchObject({
      size: { width: 256, height: 128 },
      noiseSize: { width: 128, height: 128 },
      noiseLength: expect.any(Number),
      uri: expect.stringMatching(/^data:image\/png;base64,/),
      invalid: null,
      svg: null,
    });
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}, 40_000);
