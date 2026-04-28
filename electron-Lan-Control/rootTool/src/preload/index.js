// import { contextBridge, ipcRenderer  } from 'electron'
// import { electronAPI } from '@electron-toolkit/preload'
const { contextBridge, ipcRenderer } = require('electron');
const { electronAPI } = require('@electron-toolkit/preload');


// Custom APIs for renderer
const api = {
  InfoDataList: (ch, fn) => ipcRenderer.on(ch, (event, ...age) => fn(...age)),
  remoteView: (ip) => ipcRenderer.invoke('remoteView', ip),
  command: (list) => ipcRenderer.invoke('command', list),
  GetConfig: (config) => ipcRenderer.invoke('GetConfig', config),
  scriptResultFn: (ch1, fn1) => ipcRenderer.on(ch1, (event, ...age) => fn1(...age)),
  ScriptListFn: (ch4, fn4) => ipcRenderer.on(ch4, (event, ...age) => fn4(...age)),
  saveNewConfig: (config) => ipcRenderer.invoke('saveNewConfig', config),

}

// Use `contextBridge` APIs to expose Electron APIs to
// renderer only if context isolation is enabled, otherwise
// just add to the DOM global.
if (process.contextIsolated) {
  try {
    contextBridge.exposeInMainWorld('electron', electronAPI)
    contextBridge.exposeInMainWorld('api', api)
  } catch (error) {
    console.error(error)
  }
} else {
  window.electron = electronAPI
  window.api = api
}
