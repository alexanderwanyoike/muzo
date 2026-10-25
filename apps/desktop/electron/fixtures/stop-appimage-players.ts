import { readFile, readdir } from "node:fs/promises";
import { setTimeout as delay } from "node:timers/promises";

function signal(pid: number, value: NodeJS.Signals) {
  try {
    process.kill(-pid, value);
  } catch {
    try {
      process.kill(pid, value);
    } catch {
      /* The player may already have exited. */
    }
  }
}

async function running(pid: number): Promise<boolean> {
  const status = await readFile(`/proc/${pid}/stat`, "utf8").catch(() => "");
  return (
    status.length > 0 &&
    status.slice(status.lastIndexOf(")") + 2, status.lastIndexOf(")") + 3) !==
      "Z"
  );
}

export async function stopAppImagePlayers(
  image: string,
  originalPid: number | undefined,
): Promise<void> {
  const players = new Set<number>(originalPid ? [originalPid] : []);
  for (const entry of await readdir("/proc")) {
    if (!/^\d+$/.test(entry)) continue;
    const args = (
      await readFile(`/proc/${entry}/cmdline`, "utf8").catch(() => "")
    ).split("\0");
    if (args.some((arg) => arg.startsWith("--type="))) continue;
    if (args[0] === image) players.add(Number(entry));
    else if (args[0]?.endsWith("/muzo")) {
      // APPIMAGE survives even when the broken relaunch drops every command-line argument.
      const environment = (
        await readFile(`/proc/${entry}/environ`, "utf8").catch(() => "")
      ).split("\0");
      if (environment.includes(`APPIMAGE=${image}`)) players.add(Number(entry));
    }
  }
  for (const pid of players) signal(pid, "SIGTERM");
  for (const [duration, force] of [
    [3_000, false],
    [2_000, true],
  ] as const) {
    if (force) for (const pid of players) signal(pid, "SIGKILL");
    const deadline = Date.now() + duration;
    while (Date.now() < deadline) {
      if (!(await Promise.all([...players].map(running))).some(Boolean)) return;
      await delay(50);
    }
  }
  throw new Error("An isolated AppImage player survived test cleanup");
}
