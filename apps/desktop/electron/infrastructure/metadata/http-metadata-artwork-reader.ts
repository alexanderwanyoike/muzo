import { HttpClient } from "@tokenizer/http";
import { tokenizer, type IRangeRequestClient } from "@tokenizer/range";
import { parseFromTokenizer, selectCover } from "music-metadata";
import type { ArtworkPicture } from "../../application/interfaces/track-artwork";

const metadataBudgetBytes = 8 * 1024 * 1024;
const metadataChunkBytes = 16 * 1024;
const metadataTimeoutMilliseconds = 15_000;

export class HttpMetadataArtworkReader {
  async read(url: string): Promise<ArtworkPicture | null> {
    const client = new HttpClient(url);
    let requestedBytes = 0;
    const boundedClient: IRangeRequestClient = {
      abort: () => client.abort(),
      getResponse: async (method, range) => {
        if (!range) throw new Error("Artwork requires partial requests");
        requestedBytes += range[1] - range[0] + 1;
        if (requestedBytes > metadataBudgetBytes) throw new Error("Artwork metadata exceeds the read budget");
        const response = await client.getResponse(method, range);
        const content = response.contentRange;
        if (!content || content.firstBytePosition !== range[0] || content.lastBytePosition === undefined || content.lastBytePosition > range[1]) throw new Error("Artwork server did not honor the requested range");
        return { ...response, acceptPartialRequests: true };
      },
    };
    const timeout = setTimeout(() => client.abort(), metadataTimeoutMilliseconds);
    try {
      const source = await tokenizer(boundedClient, { avoidHeadRequests: true, minimumChunkSize: metadataChunkBytes });
      try {
        return selectCover((await parseFromTokenizer(source, { duration: false })).common.picture);
      } finally { await source.close(); }
    } finally { clearTimeout(timeout); client.abort(); }
  }
}
