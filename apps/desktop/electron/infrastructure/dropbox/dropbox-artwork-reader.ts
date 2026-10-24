import type {
  DropboxArtworkReader,
  ArtworkPicture,
} from "../../application/interfaces/track-artwork";
import type { DropboxApiClient } from "./dropbox-api-client";
import type { HttpMetadataArtworkReader } from "../metadata/http-metadata-artwork-reader";

export class DropboxMetadataArtworkReader implements DropboxArtworkReader {
  constructor(
    private readonly client: Pick<DropboxApiClient, "temporaryLink">,
    private readonly metadata: Pick<HttpMetadataArtworkReader, "read">,
  ) {}
  async read(
    accessToken: string,
    path: string,
  ): Promise<ArtworkPicture | null> {
    return this.metadata.read(
      await this.client.temporaryLink(accessToken, path),
    );
  }
}
