// @vitest-environment node
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtemp, mkdir, open, readFile, readlink, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { expect, it } from "vitest";
import { previousAppImage } from "./fixtures/previous-appimage";
import { waitForRenderer } from "./fixtures/renderer-debugger";

it.skipIf(process.platform !== "linux" || !process.env.MUZO_TEST_APPIMAGE)(
  "reopens a real updated AppImage with its library, launch arguments and application menu intact",
  async () => {
    const root = await mkdtemp(join(tmpdir(), "muzo-real-update-"));
    const desktop = resolve(dirname(fileURLToPath(import.meta.url)), "..");
    const payload = resolve(process.env.MUZO_TEST_APPIMAGE ?? "");
    const profile = join(root, "config/Muzo");
    const log = await open(join(root, "launch.log"), "a");
    const previous = await previousAppImage(root, desktop, payload);
    const library = join(root, "music");
    await mkdir(library);
    const player = spawn(
      previous.image,
      [
        "--no-sandbox",
        "--disable-gpu",
        `--user-data-dir=${profile}`,
        "--remote-debugging-port=0",
      ],
      {
        env: {
          ...process.env,
          ELECTRON_RUN_AS_NODE: undefined,
          NODE_OPTIONS: undefined,
          XDG_DATA_HOME: join(root, "data"),
          XDG_CONFIG_HOME: join(root, "config"),
          XDG_CACHE_HOME: join(root, "cache"),
          ...(process.env.MUZO_TEST_FUSE === "1"
            ? {}
            : { APPIMAGE_EXTRACT_AND_RUN: "1" }),
        },
        detached: true,
        stdio: ["ignore", log.fd, log.fd],
      },
    );
    let browser: Awaited<ReturnType<typeof waitForRenderer>> | undefined;
    try {
      const portFile = join(profile, "DevToolsActivePort");
      await expect
        .poll(() => readFile(portFile, "utf8").catch(() => ""), {
          timeout: 20_000,
        })
        .not.toBe("");
      browser = await waitForRenderer(profile);
      if (!browser) throw new Error("Original window missing");
      await expect
        .poll(() => browser?.evaluate("Boolean(window.__MUZO_RUNTIME__)"), {
          timeout: 15_000,
        })
        .toBe(true);
      expect(
        await browser.evaluate(
          `window.__MUZO_RUNTIME__.invoke('add_library', {input: {name: 'Update survivor', kind: 'filesystem', location: ${JSON.stringify(library)}}})`,
        ),
      ).toMatchObject({ name: "Update survivor" });
      expect(
        await browser.evaluate(
          "window.__MUZO_RUNTIME__.updates.command('check')",
        ),
      ).toMatchObject({ ok: true });
      expect(
        await browser.evaluate(
          "window.__MUZO_RUNTIME__.updates.command('download')",
        ),
      ).toMatchObject({ ok: true });
      await expect
        .poll(
          () =>
            browser?.evaluate(
              "document.querySelector('.update-notice button')?.getAttribute('aria-label')",
            ),
          { timeout: 10_000 },
        )
        .toBe(`Restart to install v${previous.version}`);
      const originalDebugger = await readFile(portFile, "utf8");
      const originalHash = createHash("sha256")
        .update(await readFile(previous.image))
        .digest("hex");
      // Installation exits this renderer, so its IPC reply need not arrive.
      void browser
        .evaluate("document.querySelector('.update-notice button').click()")
        .catch(() => undefined);
      await expect
        .poll(() => player.exitCode, { timeout: 15_000 })
        .not.toBe(null);
      const expected = createHash("sha256")
        .update(await readFile(payload))
        .digest("hex");
      expect(expected).not.toBe(originalHash);
      expect(
        createHash("sha256")
          .update(await readFile(previous.image))
          .digest("hex"),
      ).toBe(expected);
      browser.close();
      browser = undefined;
      await expect
        .poll(() => readFile(portFile, "utf8"), { timeout: 20_000 })
        .not.toBe(originalDebugger);
      browser = await waitForRenderer(profile);
      if (!browser) throw new Error("Updated window missing");
      await expect
        .poll(() => browser?.evaluate("Boolean(window.__MUZO_RUNTIME__)"), {
          timeout: 15_000,
        })
        .toBe(true);
      expect(
        await browser.evaluate(
          "window.__MUZO_RUNTIME__.invoke('list_libraries')",
        ),
      ).toContainEqual(
        expect.objectContaining({ name: "Update survivor", location: library }),
      );
      await expect
        .poll(() => browser?.evaluate("document.body.textContent"), {
          timeout: 15_000,
        })
        .toContain("Update survivor");
      expect(await browser.evaluate("navigator.userAgent")).toContain(
        `Muzo/${previous.version}`,
      );
      const entry = await readFile(
        join(root, "data/applications/muzo.desktop"),
        "utf8",
      );
      expect(entry).toContain(`Exec="${previous.image}"`);
      expect(entry).not.toContain(".mount_");
    } catch (error) {
      throw new Error(
        `Real AppImage update failed: ${(await readFile(join(root, "launch.log"), "utf8")).slice(-5000)}`,
        { cause: error },
      );
    } finally {
      if (browser) {
        await browser.evaluate("window.close()").catch(() => undefined);
        browser.close();
      }
      if (player.pid) {
        try {
          process.kill(-player.pid, "SIGTERM");
        } catch {
          /* The old image may already have exited. */
        }
      }
      const lock = await readlink(join(profile, "SingletonLock")).catch(
        () => "",
      );
      const pid = Number(lock.split("-").at(-1));
      if (Number.isInteger(pid) && pid > 0) {
        try {
          process.kill(pid, "SIGTERM");
        } catch {
          /* The replacement may already have exited. */
        }
      }
      previous.close();
      await log.close();
      await rm(root, {
        recursive: true,
        force: true,
        maxRetries: 10,
        retryDelay: 100,
      });
    }
  },
  180_000,
);
