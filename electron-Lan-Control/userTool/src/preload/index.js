import { contextBridge, ipcRenderer } from 'electron'
import { electronAPI } from '@electron-toolkit/preload'

// Custom APIs for renderer
const api = {
  InfoDataList: (ch, fn) => ipcRenderer.on(ch, (event, ...age) => fn(...age)),
  Sendtips: (ch1, fn1) => ipcRenderer.on(ch1, (event, ...age) => fn1(...age)),
  Sendinfo: (ch2, fn2) => ipcRenderer.on(ch2, (event, ...age) => fn2(...age)),
  defaultConfigFn: (ch3, fn3) => ipcRenderer.on(ch3, (event, ...age) => fn3(...age)),
  command: (list) => ipcRenderer.invoke('command', list),
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
