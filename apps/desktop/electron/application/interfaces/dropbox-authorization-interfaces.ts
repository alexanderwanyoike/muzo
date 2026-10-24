export interface DropboxCredentials {
  accessToken: string;
  refreshToken: string | null;
  expiresIn: number;
  accountId: string | null;
}

export interface DropboxAuthorizationClient {
  exchangeAuthorizationCode(input: {
    authorizationCode: string;
    codeVerifier: string;
  }): Promise<DropboxCredentials>;
  refreshAccessToken(refreshToken: string): Promise<DropboxCredentials>;
}
