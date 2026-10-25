const { contextBridge, ipcRenderer } = require("electron");

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
    getStatus: () => ipcRenderer.invoke("muzo:get-update-status"),
    command: (command) => ipcRenderer.invoke("muzo:update-command", command),
    subscribe: (listener) => {
      const report = (_event, status) => listener(status);
      ipcRenderer.on("muzo:update-status", report);
      return () => ipcRenderer.removeListener("muzo:update-status", report);
    },
  },
});
