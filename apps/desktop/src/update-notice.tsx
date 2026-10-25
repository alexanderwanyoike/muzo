import { Download, LoaderCircle, RefreshCw, RotateCcw } from "lucide-react";
import { useEffect, useState } from "react";
import type { UpdateBridge } from "../shared/update-bridge";
import type { UpdateStatus } from "../shared/update-status";
import "./update-notice.css";

export default function UpdateNotice({ bridge }: { bridge?: UpdateBridge }) {
  const [status, setStatus] = useState<UpdateStatus>({ state: "idle" });
  const [pending, setPending] = useState(false);
  useEffect(() => {
    if (!bridge) return undefined;
    let active = true;
    let received = false;
    const unsubscribe = bridge.subscribe((next) => { received = true; if (active) setStatus(next); });
    void bridge.getStatus().then((result) => { if (active && !received && result.ok) setStatus(result.value); });
    return () => { active = false; unsubscribe(); };
  }, [bridge]);
  if (!bridge || status.state === "idle" || status.state === "checking" || status.state === "current") return null;
  let label: string;
  let detail: string;
  let command: "download" | "install" | "check";
  let Icon = Download;
  switch (status.state) {
    case "available":
      label = `Update available v${status.version}`;
      detail = status.automatic ? "Download update" : "Download installer";
      command = "download"; break;
    case "downloading":
      label = `Downloading v${status.version} · ${Math.round(status.progress)}%`;
      detail = "Keep listening. We’ll let you know when it’s ready.";
      command = "download"; Icon = LoaderCircle; break;
    case "ready":
      label = `Restart to install v${status.version}`;
      detail = "Your update is ready";
      command = "install"; Icon = RefreshCw; break;
    case "installing":
      label = `Installing v${status.version}`; detail = "Muzo will reopen shortly";
      command = "install"; Icon = LoaderCircle; break;
    case "failed":
      label = "Update failed. Retry"; detail = status.message;
      command = "check"; Icon = RotateCcw; break;
  }
  async function act() {
    if (!bridge) return;
    setPending(true);
    try {
      const result = await bridge.command(command);
      if (!result.ok) setStatus({ state: "failed", message: result.error.message });
    } finally { setPending(false); }
  }
  return (
    <div className="update-notice" aria-live="polite">
      <button type="button" aria-label={label} title={detail} disabled={pending || status.state === "downloading" || status.state === "installing"} onClick={() => { void act(); }}>
        <Icon aria-hidden="true" />
        <span><strong>{label}</strong><small>{detail}</small></span>
      </button>
    </div>
  );
}
