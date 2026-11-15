import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { setTimeout as delay } from "node:timers/promises";

const startupTimeoutMs = 20_000;
const requestTimeoutMs = 10_000;
const pollMs = 250;

export async function connectToRenderer(profile) {
  const deadline = Date.now() + startupTimeoutMs;
  let failure;
  while (Date.now() < deadline) {
    try {
      const contents = await readFile(
        join(profile, "DevToolsActivePort"),
        "utf8",
      );
      const [port] = contents.trim().split(/\r?\n/u);
      assert.match(port, /^\d+$/u);
      const response = await fetch(`http://127.0.0.1:${port}/json/list`, {
        signal: AbortSignal.timeout(requestTimeoutMs),
      });
      const targets = await response.json();
      const page = targets.find(
        ({ type, url }) => type === "page" && url.includes("index.html"),
      );
      assert(page, "The packaged Electron renderer is still opening");
      const socket = new WebSocket(page.webSocketDebuggerUrl);
      await new Promise((resolve, reject) => {
        const timer = setTimeout(() => {
          socket.close();
          reject(new Error("The renderer debugger did not connect"));
        }, requestTimeoutMs);
        socket.addEventListener(
          "open",
          () => {
            clearTimeout(timer);
            resolve();
          },
          { once: true },
        );
        socket.addEventListener(
          "error",
          () => {
            clearTimeout(timer);
            reject(new Error("The renderer debugger did not connect"));
          },
          { once: true },
        );
      });
      return socket;
    } catch (error) {
      failure = error;
      await delay(pollMs);
    }
  }
  throw new Error("The packaged Electron renderer did not become available", {
    cause: failure,
  });
}

export function renderer(socket) {
  let sequence = 0;
  return (expression) =>
    new Promise((resolve, reject) => {
      const id = ++sequence;
      const timer = setTimeout(() => {
        socket.removeEventListener("message", receive);
        reject(new Error("The renderer evaluation timed out"));
      }, requestTimeoutMs);
      function receive({ data }) {
        const message = JSON.parse(data);
        if (message.id !== id) return;
        clearTimeout(timer);
        socket.removeEventListener("message", receive);
        if (message.error || message.result?.exceptionDetails) {
          reject(new Error(`Renderer evaluation failed: ${data}`));
        } else {
          resolve(message.result?.result?.value);
        }
      }
      socket.addEventListener("message", receive);
      socket.send(
        JSON.stringify({
          id,
          method: "Runtime.evaluate",
          params: { expression, awaitPromise: true, returnByValue: true },
        }),
      );
    });
}
