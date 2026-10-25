// @vitest-environment node
import { expect, it, vi } from "vitest";
import { createApplicationUpdates } from "./application/application-updates";
import { success } from "./application/result";
import { installApplicationUpdates } from "./install-application-updates";

it("limits update commands to known player frames and broadcasts status without accepting a feed or version", async () => {
  const backend = {
    check: vi.fn(async () => success({ version: "0.0.3", automatic: true })),
    download: vi.fn(async () => success(undefined)),
    install: vi.fn(async () => success("installing" as const)),
    openRelease: vi.fn(),
    subscribe: () => () => {},
  };
  const updates = createApplicationUpdates({
    installedVersion: "0.0.2",
    backend,
  });
  const handlers = new Map<
    string,
    (event: unknown, ...args: unknown[]) => Promise<unknown>
  >();
  const send = vi.fn();
  const frame = {};
  const event = { senderFrame: frame, sender: { mainFrame: frame } };
  const window = {
    isDestroyed: () => false,
    webContents: { isDestroyed: () => false, send },
  };
  const stop = installApplicationUpdates(
    {
      handle: (channel, handler) =>
        handlers.set(
          channel,
          handler as (event: unknown, ...args: unknown[]) => Promise<unknown>,
        ),
    },
    updates,
    () => window,
    () => [window],
  );
  const command = handlers.get("muzo:update-command")!;
  expect(await command({ ...event, senderFrame: {} }, "check")).toMatchObject({
    ok: false,
  });
  expect(
    await command(event, "check", { feed: "https://evil.test" }),
  ).toMatchObject({ ok: false });
  expect(backend.check).not.toHaveBeenCalled();
  expect(await command(event, "check")).toMatchObject({ ok: true });
  expect(send).toHaveBeenLastCalledWith("muzo:update-status", {
    state: "available",
    version: "0.0.3",
    automatic: true,
  });
  expect(await handlers.get("muzo:get-update-status")!(event)).toEqual(
    success(updates.getStatus()),
  );
  stop();
  send.mockClear();
  await updates.download();
  expect(send).not.toHaveBeenCalled();
});
