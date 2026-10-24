const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld("__MUZO_RUNTIME__", {
  invoke: async (command, args) => {
    const result = await ipcRenderer.invoke("muzo:invoke", command, args);
    if (result.ok) {
      return result.value;
    }
    throw result.error;
  },
  openDirectory: () => ipcRenderer.invoke("muzo:open-directory"),
});
