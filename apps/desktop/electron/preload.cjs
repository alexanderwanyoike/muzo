const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld("__MUZO_RUNTIME__", {
  invoke: (command, args) => ipcRenderer.invoke("muzo:invoke", command, args),
  openDirectory: () => ipcRenderer.invoke("muzo:open-directory"),
});
