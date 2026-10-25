import { useEffect, useId, useState } from "react";
import type { UpdateBridge } from "../shared/update-bridge";
import type { UpdateStatus } from "../shared/update-status";
import "./update-notice.css";

export default function UpdateNotice({ bridge }: { bridge?: UpdateBridge }) {
  const descriptionId = useId();
  const [status, setStatus] = useState<UpdateStatus>({ state: "idle" });
  const [pending, setPending] = useState(false);
  useEffect(() => {
    if (!bridge) return undefined;
    let active = true;
    let received = false;
    const unsubscribe = bridge.subscribe((next) => {
      received = true;
      if (active) setStatus(next);
    });
    void bridge.getStatus().then((result) => {
      if (active && !received && result.ok) setStatus(result.value);
    });
    return () => {
      active = false;
      unsubscribe();
    };
  }, [bridge]);
  if (!bridge) return null;
  let label: string;
  let detail: string;
  let command: "download" | "install" | "check";
  switch (status.state) {
    case "idle":
      label = "Check for updates";
      detail = "Check for a new Muzo version";
      command = "check";
      break;
    case "checking":
      label = "Checking for updates...";
      detail = "Checking for a new Muzo version";
      command = "check";
      break;
    case "current":
      label = "Up to date";
      detail = "Check for updates";
      command = "check";
      break;
    case "available":
      label = `Update available to ${status.version}`;
      detail = status.automatic ? "Download update" : "Download installer";
      command = "download";
      break;
    case "downloading":
      label = `Downloading v${status.version} · ${Math.round(status.progress)}%`;
      detail = "Keep listening. We’ll let you know when it’s ready.";
      command = "download";
      break;
    case "ready":
      label = `Restart to install v${status.version}`;
      detail = "Your update is ready";
      command = "install";
      break;
    case "installing":
      label = `Installing v${status.version}`;
      detail = "Muzo will reopen shortly";
      command = "install";
      break;
    case "failed":
      label = "Update failed. Retry";
      detail = status.message;
      command = "check";
      break;
  }
  async function act() {
    if (!bridge) return;
    setPending(true);
    try {
      const result = await bridge.command(command);
      if (!result.ok)
        setStatus({ state: "failed", message: result.error.message });
    } finally {
      setPending(false);
    }
  }
  return (
    <div className="update-notice" aria-live="polite" aria-atomic="true">
      <button
        type="button"
        aria-label={label}
        aria-describedby={descriptionId}
        title={detail}
        disabled={
          pending ||
          status.state === "checking" ||
          status.state === "downloading" ||
          status.state === "installing"
        }
        onClick={() => {
          void act();
        }}
      >
        <span>{label}</span>
      </button>
      <span id={descriptionId} hidden>
        {detail}
      </span>
    </div>
  );
}
