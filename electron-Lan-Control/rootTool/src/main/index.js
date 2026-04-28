import { app, shell, BrowserWindow, ipcMain } from 'electron'
import { join } from 'path'
import { electronApp, optimizer, is } from '@electron-toolkit/utils'
import icon from '../../resources/icon.png?asset'
import { runInit, runShutdown } from './lifecycle'
import { initContext, setWindow } from './modules/config/getConfig'
import { sendToRenderer } from './modules/logger'
import createListeners from './lifecycle/windowListeners'
// 导入ipc
import './modules/ipc'

const FilePath = join(__dirname, '../../resources').replace('app.asar', 'app.asar.unpacked')
const gotTheLock = app.requestSingleInstanceLock()
let ListenersMap = null // 窗口监听器映射
let initialized = false // 防止多次创建窗口

function createWindow() {
  // Create the browser window.
  const win = new BrowserWindow({
    width: 900,
    height: 670,
    show: false,
    autoHideMenuBar: true,
    frame: false,
    ...(process.platform === 'linux' ? { icon } : {}),
    webPreferences: {
      preload: join(__dirname, '../preload/index.js'),
      sandbox: false,
      nodeIntegration: process.env.ELECTRON_NODE_INTEGRATION,
      contextIsolation: !process.env.ELECTRON_NODE_INTEGRATION,
      nodeIntegration: true,
      devTools: true
      // webSecurity: false
    }
  })

  // 设置win
  setWindow(win)
  win.once('ready-to-show', () => {
    if (!initialized) {
      // 初始化context.js
      initContext(win, FilePath)
      // 初始化监听广播端口和被控端名称/执行启动相关
      runInit(FilePath)
      win.show()
      initialized = true
    }
  })

  win.webContents.setWindowOpenHandler((details) => {
    shell.openExternal(details.url)
    return { action: 'deny' }
  })

  // HMR for renderer base on electron-vite cli.
  // Load the remote URL for development or the local html file for production.
  if (is.dev && process.env['ELECTRON_RENDERER_URL']) {
    win.loadURL(process.env['ELECTRON_RENDERER_URL'])
    win.webContents.openDevTools({ mode: 'detach' })
  } else {
    win.loadFile(join(__dirname, '../renderer/index.html'))
  }

  if (!ListenersMap) {
    ListenersMap = createListeners(win, runShutdown)
    Object.keys(ListenersMap).forEach((key) => {
      ipcMain.on(key, () => {
        ListenersMap[key]()
      })
    })
  }

  // 捕获未捕获的异常
  process.on('uncaughtException', (err) => {
    win && win.webContents.openDevTools({ mode: 'detach' })
    sendToRenderer('scriptResult', {
      sta: false,
      text: `主进程未捕获异常: ${err.message}`,
      title: '主进程异常'
    })
  })
}
// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
app.whenReady().then(() => {
  // Set app user model id for windows
  electronApp.setAppUserModelId('com.electron')

  // Default open or close DevTools by F12 in development
  // and ignore CommandOrControl + R in production.
  // see https://github.com/alex8088/electron-toolkit/tree/master/packages/utils
  app.on('browser-window-created', (_, window) => {
    optimizer.watchWindowShortcuts(window)
  })

  createWindow()

  app.on('activate', function () {
    // On macOS it's common to re-create a window in the app when the
    // dock icon is clicked and there are no other windows open.
    if (BrowserWindow.getAllWindows().length === 0) createWindow()
  })
})

// Quit when all windows are closed, except on macOS. There, it's common
// for applications and their menu bar to stay active until the user quits
// explicitly with Cmd + Q.
app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit()
  }
})

// 单实例应用
if (!gotTheLock) {
  // 如果获取失败，说明已有实例在运行，直接退出当前进程
  app.quit()
} else {
  // 如果是主实例，监听第二次启动事件
  app.on('second-instance', (event, commandLine, workingDirectory) => {
    // 如果已有窗口，激活并显示
    if (win) {
      if (win.isMinimized()) win.restore()
      win.focus()
    }
  })
}

// In this file you can include the rest of your app's specific main process
// code. You can also put them in separate files and require them here.
