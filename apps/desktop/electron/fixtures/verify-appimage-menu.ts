import assert from "node:assert/strict";
import { execFile, spawn } from "node:child_process";
import { mkdir, mkdtemp, readFile, readdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { promisify } from "node:util";

function signalGroup(pid: number, signal: NodeJS.Signals): void {
  try {
    process.kill(-pid, signal);
  } catch (error) {
    if (!(error instanceof Error && "code" in error && error.code === "ESRCH")) throw error;
  }
}

const imageArgument = process.argv[2];
assert(imageArgument, "An AppImage path is required");
const image = resolve(imageArgument);
const run = promisify(execFile);

for (const repair of [false, true]) {
  const directory = await mkdtemp(join(tmpdir(), "muzo-appimage-menu-"));
  const data = join(directory, "data");
  const filename = repair ? "Muzo.cinnamon-generated.desktop" : "muzo.desktop";
  const launcher = join(data, "applications", filename);
  if (repair) {
    await mkdir(join(data, "applications"), { recursive: true });
    await writeFile(launcher, "[Desktop Entry]\nType=Application\nName=Muzo\nExec=/tmp/.mount_Muzo-old/muzo\nTerminal=false\n");
  }
  const environment = {
    ...process.env,
    XDG_DATA_HOME: data,
    XDG_CONFIG_HOME: join(directory, "config"),
    XDG_CACHE_HOME: join(directory, "cache"),
    APPIMAGE_EXTRACT_AND_RUN: "1",
    ELECTRON_RUN_AS_NODE: undefined,
    NODE_OPTIONS: undefined,
  };
  const player = spawn(image, ["--no-sandbox", "--disable-gpu"], {
    env: environment,
    detached: true,
    stdio: "ignore",
  });
  try {
    assert(player.pid, "The AppImage did not start");
    let registered = false;
    for (let attempt = 0; attempt < 100; attempt += 1) {
      const entry = await readFile(launcher, "utf8").catch(() => "");
      if (entry.includes("X-Muzo-AppImage-Integration=true")) {
        registered = true;
        break;
      }
      assert.equal(player.exitCode, null, "The AppImage exited before registration");
      await delay(100);
    }
    assert(registered, "First AppImage launch did not register its application menu entry");
    await run("desktop-file-validate", [launcher]);
    await run("/usr/bin/python3", ["-c", [
      "from gi.repository import Gio",
      "import os,shlex,sys",
      "app=Gio.DesktopAppInfo.new(sys.argv[1])",
      "assert app.get_name()=='Muzo'",
      "assert app.get_startup_wm_class()=='muzo'",
      "assert shlex.split(app.get_commandline())[0]==sys.argv[2]",
      "assert os.path.isfile(app.get_icon().get_file().get_path())",
    ].join("\n"), filename, image], { env: environment });
    const files = await readdir(join(data, "applications"));
    assert.deepEqual(files, [filename], "Registration created duplicate menu entries");
    const { stdout } = await run("xwininfo", ["-root", "-tree"]);
    assert.match(stdout, /"Muzo": \("muzo" "muzo"\)/, "Window identity does not match its launcher");
    process.stdout.write(`${repair ? "Cinnamon launcher repair" : "First AppImage launch"}: native menu, icon, stable executable and window identity verified\n`);
  } finally {
    if (player.pid) {
      const closed = new Promise<void>((done) => player.once("close", () => done()));
      signalGroup(player.pid, "SIGTERM");
      await Promise.race([closed, delay(2_000)]);
      signalGroup(player.pid, "SIGKILL");
    }
    await rm(directory, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });
  }
}
