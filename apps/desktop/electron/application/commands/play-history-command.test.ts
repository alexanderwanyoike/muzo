// @vitest-environment node

import { describe, expect, it, vi } from "vitest";

import {
  ListTrackPlayCountsCommand,
  RecordTrackPlayCommand,
} from "./play-history-command";

describe("RecordTrackPlayCommand", () => {
  it("records a play for the requested track", async () => {
    const playHistory = {
      recordPlay: vi.fn().mockResolvedValue(undefined),
    };

    await expect(
      new RecordTrackPlayCommand(
        playHistory,
        () => "play-1",
        () => 1_719_000_000,
      ).handle({
        input: { libraryId: "lib-1", trackId: "trk-1" },
      }),
    ).resolves.toBeUndefined();

    expect(playHistory.recordPlay).toHaveBeenCalledWith({
      id: "play-1",
      libraryId: "lib-1",
      trackId: "trk-1",
      playedAtUnixSeconds: 1_719_000_000,
    });
  });
});

describe("ListTrackPlayCountsCommand", () => {
  it("lists play counts for the requested library", async () => {
    const counts = [
      {
        trackId: "trk-1",
        playCount: 2,
        lastPlayedAtUnixSeconds: 1_719_000_100,
      },
    ];
    const playHistory = {
      listPlayCounts: vi.fn().mockResolvedValue(counts),
    };

    await expect(
      new ListTrackPlayCountsCommand(playHistory).handle({
        input: { libraryId: "lib-1" },
      }),
    ).resolves.toEqual(counts);

    expect(playHistory.listPlayCounts).toHaveBeenCalledWith("lib-1");
  });
});
