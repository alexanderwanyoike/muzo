// @vitest-environment node

import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

import { describe, expect, it } from "vitest";

import { NodeAudioFileWalker } from "./node-audio-file-walker";

describe("NodeAudioFileWalker", () => {
  it("walks audio files recursively and ignores non-audio files", async () => {
    const root = tempDirectory();
    mkdirSync(join(root, "nested"), { recursive: true });
    writeFileSync(join(root, "song.mp3"), "audio");
    writeFileSync(join(root, "nested", "voice.WAV"), "audio");
    writeFileSync(join(root, "cover.jpg"), "image");

    const files = await new NodeAudioFileWalker().walkAudioFiles(root);

    expect(files.map((file) => file.path).sort()).toEqual([
      join(root, "nested", "voice.WAV"),
      join(root, "song.mp3"),
    ]);
    expect(files.every((file) => file.size > 0)).toBe(true);
    expect(files.every((file) => file.mtime > 0)).toBe(true);
  });
});

function tempDirectory(): string {
  return join(tmpdir(), `muzo-${Date.now()}-${Math.random()}`);
}
