import { afterEach, expect, it, vi } from "vitest";
import { act, cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { UpdateStatus } from "../shared/update-status";
import { success } from "../electron/application/result";
import UpdateNotice from "./update-notice";

afterEach(cleanup);
it("offers a status bar download, shows progress and waits for a separate restart click", async () => {
  let report: ((status: UpdateStatus) => void) | undefined;
  const unsubscribe = vi.fn();
  const bridge = {
    getStatus: async () =>
      success<UpdateStatus>({
        state: "available",
        version: "0.0.3",
        automatic: true,
      }),
    subscribe: (listener: (status: UpdateStatus) => void) => {
      report = listener;
      return unsubscribe;
    },
    command: vi.fn(async () => success(undefined)),
  };
  const { unmount } = render(<UpdateNotice bridge={bridge} />);
  await userEvent.click(
    await screen.findByRole("button", { name: "Update available to 0.0.3" }),
  );
  expect(bridge.command).toHaveBeenCalledOnce();
  expect(bridge.command).toHaveBeenCalledWith("download");
  act(() =>
    report?.({ state: "downloading", version: "0.0.3", progress: 42.7 }),
  );
  expect(
    screen
      .getByRole("button", { name: "Downloading v0.0.3 · 43%" })
      .hasAttribute("disabled"),
  ).toBe(true);
  act(() => report?.({ state: "ready", version: "0.0.3" }));
  expect(bridge.command).not.toHaveBeenCalledWith("install");
  await userEvent.click(
    screen.getByRole("button", { name: "Restart to install v0.0.3" }),
  );
  expect(bridge.command).toHaveBeenLastCalledWith("install");
  unmount();
  expect(unsubscribe).toHaveBeenCalledOnce();
});
it("keeps unsigned platform downloads honest and makes failed updates retryable", async () => {
  let report: ((status: UpdateStatus) => void) | undefined;
  const bridge = {
    getStatus: async () =>
      success<UpdateStatus>({
        state: "available",
        version: "0.0.3",
        automatic: false,
      }),
    subscribe: (listener: (status: UpdateStatus) => void) => {
      report = listener;
      return () => {};
    },
    command: vi.fn(async () => success(undefined)),
  };
  render(<UpdateNotice bridge={bridge} />);
  const notice = await screen.findByRole("button", {
    name: "Update available to 0.0.3",
  });
  expect(notice.getAttribute("title")).toBe("Download installer");
  await userEvent.click(notice);
  expect(bridge.command).toHaveBeenCalledWith("download");
  act(() => report?.({ state: "failed", message: "Could not download." }));
  await userEvent.click(
    screen.getByRole("button", { name: "Update failed. Retry" }),
  );
  expect(bridge.command).toHaveBeenLastCalledWith("check");
});

it("shows checking progress and allows a fresh check when the app is up to date", async () => {
  let report: ((status: UpdateStatus) => void) | undefined;
  const bridge = {
    getStatus: async () => success<UpdateStatus>({ state: "checking" }),
    subscribe: (listener: (status: UpdateStatus) => void) => {
      report = listener;
      return () => {};
    },
    command: vi.fn(async () => success(undefined)),
  };
  render(<UpdateNotice bridge={bridge} />);
  expect(
    (
      await screen.findByRole("button", { name: "Checking for updates..." })
    ).hasAttribute("disabled"),
  ).toBe(true);
  act(() => report?.({ state: "current" }));
  await userEvent.click(screen.getByRole("button", { name: "Up to date" }));
  expect(bridge.command).toHaveBeenCalledWith("check");
});
