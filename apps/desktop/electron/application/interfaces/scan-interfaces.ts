export interface WalkedAudioFile {
  path: string;
  size: number;
  mtime: number;
}

export interface AudioMetadata {
  title: string;
  artist: string;
  album: string | null;
  trackNumber: number | null;
  discNumber: number | null;
  genre: string | null;
  year: number | null;
  durationSeconds: number;
}

export interface AudioFileWalker {
  walkAudioFiles(root: string): Promise<WalkedAudioFile[]>;
}

export interface AudioMetadataReader {
  read(path: string): Promise<AudioMetadata>;
}
