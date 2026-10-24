import type { nativeImage } from "electron";
import type { ArtworkPicture, ArtworkThumbnails } from "../../application/interfaces/track-artwork";

const maximumArtworkBytes = 8 * 1024 * 1024;
const thumbnailEdge = 256;
const maximumThumbnailLength = 256 * 1024;
const rasterFormats = new Set(["image/png", "image/jpeg", "image/jpg", "image/webp"]);

export class ElectronArtworkThumbnails implements ArtworkThumbnails {
  constructor(private readonly images: Pick<typeof nativeImage, "createFromBuffer">) {}

  create(picture: ArtworkPicture): string | null {
    if (!rasterFormats.has(picture.format.toLowerCase()) || picture.data.byteLength > maximumArtworkBytes) return null;
    const image = this.images.createFromBuffer(Buffer.from(picture.data));
    if (image.isEmpty()) return null;
    const size = image.getSize();
    const scale = Math.min(1, thumbnailEdge / Math.max(size.width, size.height));
    const thumbnail = image.resize({ width: Math.max(1, Math.round(size.width * scale)), height: Math.max(1, Math.round(size.height * scale)), quality: "good" }).toDataURL();
    return thumbnail.length <= maximumThumbnailLength ? thumbnail : null;
  }
}
