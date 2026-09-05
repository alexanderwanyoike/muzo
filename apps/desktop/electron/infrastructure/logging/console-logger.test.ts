// @vitest-environment node

import { afterEach, describe, expect, it, vi } from "vitest";

import { ConsoleLogger } from "./console-logger";

describe("ConsoleLogger", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("writes error messages to stderr", () => {
    const error = vi.spyOn(console, "error").mockImplementation(() => undefined);

    new ConsoleLogger().error("watch failed");

    expect(error).toHaveBeenCalledWith("watch failed");
  });
});
