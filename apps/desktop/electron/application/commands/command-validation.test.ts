// @vitest-environment node
import { expect, it, vi } from "vitest";
import { AddLibraryCommand } from "./add-library-command";
import {
  CreatePlaylistCommand,
  AddTrackToPlaylistCommand,
  RemovePlaylistEntryCommand,
  ReorderPlaylistEntriesCommand,
} from "./playlist-command";
import {
  RecordTrackPlayCommand,
  ListTrackPlayCountsCommand,
} from "./play-history-command";
import {
  EditTrackMetadataCommand,
  ClearTrackMetadataOverrideCommand,
} from "./track-metadata-command";
import { ScanLibraryCommand } from "./scan-library-command";
import { PrepareTrackAudioSourceCommand } from "./prepare-track-audio-source-command";
import { ListLibrariesCommand } from "./list-libraries-command";
import { ListTracksCommand } from "./list-tracks-command";

const playlists = {
  create: vi.fn(),
  list: vi.fn(),
  addTrack: vi.fn(),
  removeEntry: vi.fn(),
  reorderEntries: vi.fn(),
};
const tracks = {
  updateMetadataOverride: vi.fn(),
  clearMetadataOverride: vi.fn(),
};
const history = { listPlayCounts: vi.fn(), recordPlay: vi.fn() };
const commands = [
  new AddLibraryCommand(
    { add: vi.fn(), list: vi.fn(), findById: vi.fn() },
    () => "id",
    { watchLibrary: vi.fn() },
    { scan: vi.fn() },
    { error: vi.fn() },
  ),
  new CreatePlaylistCommand(playlists),
  new AddTrackToPlaylistCommand(playlists),
  new RemovePlaylistEntryCommand(playlists),
  new ReorderPlaylistEntriesCommand(playlists),
  new RecordTrackPlayCommand(
    history,
    () => "id",
    () => 1000,
  ),
  new ListTrackPlayCountsCommand(history),
  new EditTrackMetadataCommand(tracks),
  new ClearTrackMetadataOverrideCommand(tracks),
  new ScanLibraryCommand({ scan: vi.fn() }),
  new PrepareTrackAudioSourceCommand({ prepare: vi.fn() }),
  new ListTracksCommand({ listForLibrary: vi.fn() }),
];
it.each(commands)(
  "returns a typed input failure for $command",
  async (command) => {
    expect(await command.handle(undefined)).toMatchObject({
      ok: false,
      error: { kind: "repository" },
    });
  },
);

it("returns a typed storage failure when a library query fails", async () => {
  const command = new ListLibrariesCommand({
    list: vi.fn().mockRejectedValue(new Error("database unavailable")),
    add: vi.fn(),
    findById: vi.fn(),
  });
  expect(await command.handle()).toMatchObject({
    ok: false,
    error: { kind: "repository" },
  });
});
