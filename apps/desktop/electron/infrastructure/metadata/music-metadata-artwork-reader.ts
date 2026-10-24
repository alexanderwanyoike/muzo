import { parseFile, selectCover } from "music-metadata";
import type { FileArtworkReader, ArtworkPicture } from "../../application/interfaces/track-artwork";

export class MusicMetadataArtworkReader implements FileArtworkReader {
  async read(path: string): Promise<ArtworkPicture | null> {
    return selectCover((await parseFile(path, { duration: false })).common.picture);
  }
}
