import type { Result } from "../../../shared/result";

export interface DropboxAudioDownload {
  download(accessToken: string, trackKey: string, remotePath: string): Promise<Result<string>>;
}
