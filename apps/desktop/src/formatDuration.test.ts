import { describe, it, expect } from "vitest";
import { formatDuration } from "./formatDuration";

describe("formatDuration", () => {
  it("formats seconds under a minute as m:ss", () => {
    expect(formatDuration(5)).toBe("0:05");
    expect(formatDuration(59)).toBe("0:59");
  });

  it("formats seconds between a minute and an hour as m:ss", () => {
    expect(formatDuration(65)).toBe("1:05");
    expect(formatDuration(119)).toBe("1:59");
    expect(formatDuration(600)).toBe("10:00");
  });

  it("formats hours as h:mm:ss", () => {
    expect(formatDuration(3600)).toBe("1:00:00");
    expect(formatDuration(3665)).toBe("1:01:05");
  });

  it("handles zero", () => {
    expect(formatDuration(0)).toBe("0:00");
  });

  it("handles non-finite or negative input as 0:00", () => {
    expect(formatDuration(NaN)).toBe("0:00");
    expect(formatDuration(-1)).toBe("0:00");
    expect(formatDuration(Infinity)).toBe("0:00");
  });
});
