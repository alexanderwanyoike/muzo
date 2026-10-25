import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { setTimeout as delay } from "node:timers/promises";

export async function rendererDebugger(profile: string) {
  const [port] = (await readFile(join(profile, "DevToolsActivePort"), "utf8"))
    .trim()
    .split("\n");
  const targets = (await (
    await fetch(`http://127.0.0.1:${port}/json/list`)
  ).json()) as { type: string; webSocketDebuggerUrl: string }[];
  const target = targets.find(({ type }) => type === "page");
  if (!target) throw new Error("The player window is not ready");
  const socket = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise<void>((resolve, reject) => {
    socket.addEventListener("open", () => resolve(), { once: true });
    socket.addEventListener(
      "error",
      () => reject(new Error("Debugger unavailable")),
      { once: true },
    );
  });
  let sequence = 0;
  return {
    close: () => socket.close(),
    async evaluate(expression: string): Promise<unknown> {
      sequence += 1;
      const id = sequence;
      return new Promise((resolve, reject) => {
        const timer = setTimeout(() => {
          socket.removeEventListener("message", receive);
          reject(new Error("Renderer evaluation timed out"));
        }, 10_000);
        function receive(event: MessageEvent<string>) {
          const message = JSON.parse(event.data) as {
            id?: number;
            error?: { message: string };
            result?: {
              exceptionDetails?: unknown;
              result?: { value?: unknown };
            };
          };
          if (message.id !== id) return;
          clearTimeout(timer);
          socket.removeEventListener("message", receive);
          if (message.error || message.result?.exceptionDetails)
            reject(new Error(`Renderer evaluation failed: ${event.data}`));
          else resolve(message.result?.result?.value);
        }
        socket.addEventListener("message", receive);
        socket.send(
          JSON.stringify({
            id,
            method: "Runtime.evaluate",
            params: {
              expression,
              awaitPromise: true,
              returnByValue: true,
            },
          }),
        );
      });
    },
  };
}

export async function waitForRenderer(profile: string) {
  const deadline = Date.now() + 15_000;
  while (Date.now() < deadline) {
    try {
      return await rendererDebugger(profile);
    } catch {
      await delay(100);
    }
  }
  throw new Error("The player debugger did not become available");
}
