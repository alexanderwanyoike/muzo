import { useEffect, useState } from "react";
import type { DropboxConnectionStatus } from "../shared/dropbox-contracts";
import type { LibraryDto } from "./types";
import DropboxLibrarySettings from "./dropbox-library-settings";
import {
  beginDropboxConnection, completeDropboxConnection, disconnectDropbox, dropboxStatus,
} from "./dropbox-api";

export default function DropboxSettings({ onLibraryAdded = () => {} }: { onLibraryAdded?: (library: LibraryDto) => void }) {
  const [connection, setConnection] = useState<DropboxConnectionStatus | null>(null);
  const [clientId, setClientId] = useState("");
  const [code, setCode] = useState("");
  const [pending, setPending] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void dropboxStatus().then((result) => {
      if (cancelled) return;
      if (result.ok) setConnection(result.value);
      else setError(result.error?.message ?? "Could not read Dropbox status.");
    });
    return () => { cancelled = true; };
  }, []);

  async function begin() {
    setBusy(true);
    setError(null);
    const result = await beginDropboxConnection(clientId.trim());
    if (result.ok) setPending(true);
    else setError(result.error.message);
    setBusy(false);
  }

  async function complete() {
    setBusy(true);
    setError(null);
    const result = await completeDropboxConnection(code.trim());
    if (result.ok) {
      setConnection(result.value);
      setPending(false);
      setCode("");
    } else setError(result.error.message);
    setBusy(false);
  }

  async function disconnect() {
    setBusy(true);
    setError(null);
    const result = await disconnectDropbox();
    if (result.ok) {
      setConnection(result.value);
      setPending(false);
      setCode("");
    } else setError(result.error.message);
    setBusy(false);
  }

  return (
    <section className="settings-view__section" aria-label="Dropbox">
      <div><h3>Dropbox</h3><p>Connect your music in Dropbox.</p></div>
      {connection?.connected ? (
        <div>
          <p role="status">Connected to Dropbox.</p>
          <DropboxLibrarySettings onAdded={onLibraryAdded} />
          <button type="button" disabled={busy} onClick={() => { void disconnect(); }}>Disconnect Dropbox</button>
        </div>
      ) : (
        <div className="add-library-form">
          <label className="add-library-form__field">Dropbox app key
            <input value={clientId} onChange={(event) => setClientId(event.target.value)} autoComplete="off" disabled={busy || pending} />
          </label>
          <button type="button" disabled={busy || !clientId.trim()} onClick={() => { void begin(); }}>
            {pending ? "Restart authorization" : "Connect Dropbox"}
          </button>
          {pending && <>
            <p>Approve access in your browser, then paste the authorization code here.</p>
            <label className="add-library-form__field">Authorization code
              <input value={code} onChange={(event) => setCode(event.target.value)} autoComplete="off" disabled={busy} />
            </label>
            <button type="button" disabled={busy || !code.trim()} onClick={() => { void complete(); }}>Finish connecting</button>
            <button type="button" disabled={busy} onClick={() => { void disconnect(); }}>Cancel connection</button>
          </>}
        </div>
      )}
      {error && <p role="alert" className="app__error">{error}</p>}
    </section>
  );
}
