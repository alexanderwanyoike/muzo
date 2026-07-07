// @vitest-environment node

import { describe, expect, it } from "vitest";

import type { DropboxEntry } from "./dropbox-api-client";
import {
  DropboxLibraryCatalogAdapter,
  type DropboxFileLister,
} from "./dropbox-library-catalog";

describe("DropboxLibraryCatalogAdapter", () => {
  it("lists supported Dropbox audio files as source files", async () => {
    const adapter = new DropboxLibraryCatalogAdapter(
      new FakeDropboxFileLister([
        {
          tag: "folder",
          name: "Album",
          id: "id:folder",
          pathLower: "/music/album",
          pathDisplay: "/Music/Album",
          serverModified: null,
          rev: null,
          size: null,
          contentHash: null,
        },
        {
          tag: "file",
          name: "Song.MP3",
          id: "id:file-1",
          pathLower: "/music/album/song.mp3",
          pathDisplay: "/Music/Album/Song.MP3",
          serverModified: "2026-06-25T10:30:00Z",
          rev: "rev-1",
          size: 12_345,
          contentHash: "hash-1",
        },
        {
          tag: "file",
          name: "Cover.jpg",
          id: "id:file-2",
          pathLower: "/music/album/cover.jpg",
          pathDisplay: "/Music/Album/Cover.jpg",
          serverModified: "2026-06-25T10:31:00Z",
          rev: "rev-2",
          size: 999,
          contentHash: "hash-2",
        },
      ]),
    );

    await expect(adapter.listAudioFiles("access-123", "/Music")).resolves.toEqual(
      [
        {
          path: "/Music/Album/Song.MP3",
          size: 12_345,
          modifiedEpochSeconds: 1_782_383_400,
        },
      ],
    );
  });
});

class FakeDropboxFileLister implements DropboxFileLister {
  constructor(private readonly entries: DropboxEntry[]) {}

  async listFolderRecursive(): Promise<DropboxEntry[]> {
    return this.entries;
  }
}
