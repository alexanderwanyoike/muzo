import DropboxFolderPicker from "./dropbox-folder-picker";
import { useEffect, useRef, useState } from "react";
import { dropboxSyncStatus, setDropboxSyncInterval } from "./dropbox-api";
import type { LibraryDto } from "./types";

export default function DropboxLibrarySettings({
  onAdded,
  accountId,
}: {
  accountId: string;
  onAdded: (library: LibraryDto) => void;
}) {
  const [browsing, setBrowsing] = useState(false);
  const browseButton = useRef<HTMLButtonElement>(null);
  const wasBrowsing = useRef(false);
  useEffect(() => {
    if (wasBrowsing.current && !browsing) browseButton.current?.focus();
    wasBrowsing.current = browsing;
  }, [browsing]);
  const [minutes, setMinutes] = useState(5);
  const [savingInterval, setSavingInterval] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [syncMessage, setSyncMessage] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    const load = async () => {
      const result = await dropboxSyncStatus();
      if (cancelled) return;
      if (result.ok) {
        setMinutes(result.value.intervalMinutes ?? 5);
        setSyncMessage(
          result.value.lastError ??
            (result.value.syncing ? "Syncing Dropbox..." : null),
        );
      } else setSyncMessage(result.error.message);
    };
    void load();
    const timer = setInterval(() => {
      void load();
    }, 5000);
    return () => {
      cancelled = true;
      clearInterval(timer);
    };
  }, []);

  async function changeInterval(value: number) {
    setSavingInterval(true);
    setError(null);
    const result = await setDropboxSyncInterval(value);
    if (result.ok) setMinutes(result.value.intervalMinutes);
    else setError(result.error.message);
    setSavingInterval(false);
  }

  return (
    <div>
      {browsing ? (
        <DropboxFolderPicker
          key={accountId}
          accountId={accountId}
          onAdded={onAdded}
          onClose={() => setBrowsing(false)}
        />
      ) : (
        <button
          className="dropbox-browse-button"
          ref={browseButton}
          type="button"
          disabled={!accountId}
          onClick={() => setBrowsing(true)}
        >
          Browse Dropbox folders
        </button>
      )}
      <label>
        Sync Dropbox every
        <select
          value={minutes}
          disabled={savingInterval}
          onChange={(event) => {
            void changeInterval(Number(event.target.value));
          }}
        >
          {[1, 5, 15, 30, 60].map((value) => (
            <option key={value} value={value}>
              {value} {value === 1 ? "minute" : "minutes"}
            </option>
          ))}
        </select>
      </label>
      <p>
        Dropbox folders sync in the background. Use Refresh under Indexed
        sources to sync now.
      </p>
      {syncMessage && <p role="status">{syncMessage}</p>}
      {error && (
        <p role="alert" className="app__error">
          {error}
        </p>
      )}
    </div>
  );
}
