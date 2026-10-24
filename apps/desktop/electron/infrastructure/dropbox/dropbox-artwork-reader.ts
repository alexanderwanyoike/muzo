import type {
  DropboxArtworkReader,
  ArtworkPicture,
  HttpArtworkReader,
} from "../../application/interfaces/track-artwork";
import type { DropboxApiClient } from "./dropbox-api-client";

export class DropboxMetadataArtworkReader implements DropboxArtworkReader {
  constructor(
    private readonly client: Pick<DropboxApiClient, "temporaryLink">,
    private readonly metadata: HttpArtworkReader,
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
