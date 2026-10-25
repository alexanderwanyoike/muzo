const { contextBridge, ipcRenderer } = require("electron");

async function updateRequest(channel, ...args) {
  try {
    return await ipcRenderer.invoke(channel, ...args);
  } catch {
    return {
      ok: false,
      error: {
        kind: "remote",
        message: "The update service is unavailable. Reopen Muzo to retry.",
      },
    };
  }
}

contextBridge.exposeInMainWorld("__MUZO_RUNTIME__", {
  invoke: async (command, args) => {
    const result = await ipcRenderer.invoke("muzo:invoke", command, args);
    if (result.ok) {
      return result.value;
    }
    throw result.error;
  },
  setPlayerWindowMode: (mode) =>
    ipcRenderer.invoke("muzo:player-window-mode", mode),
  openDirectory: () => ipcRenderer.invoke("muzo:open-directory"),
  updates: {
    getStatus: () => updateRequest("muzo:get-update-status"),
    command: (command) => updateRequest("muzo:update-command", command),
    subscribe: (listener) => {
      const report = (_event, status) => listener(status);
      ipcRenderer.on("muzo:update-status", report);
      return () => ipcRenderer.removeListener("muzo:update-status", report);
    },
  },
});
