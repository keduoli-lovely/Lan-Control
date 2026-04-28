import { app, shell, BrowserWindow, ipcMain, Tray, Menu } from 'electron'
import { join } from 'path'
import { electronApp, optimizer, is } from '@electron-toolkit/utils'
import icon from '../../resources/icon.png?asse&asarUnpack'
import { startUltraVNC } from './tool/RunScript'
import { runShutdown, initAppScript } from './tool/initApp'
import { sendCommand } from '@keduoli-q/hardware-monitor'
import { handleData, initUpdateConfig } from './tool/updateConfig'
import ConfigManager from './tool/Config'

const FilePath = join(__dirname, '../../resources').replace('app.asar', 'app.asar.unpacked')
// icon
const trayIcon = join(FilePath, './icon.png')
const gotTheLock = app.requestSingleInstanceLock()

let tray

function createWindow() {
  // Create the browser window.
  const win = new BrowserWindow({
    width: 900,
    height: 670,
    show: false,
    autoHideMenuBar: true,
    frame: false,
    skipTaskbar: true,
    ...(process.platform === 'linux' ? { icon } : {}),
    webPreferences: {
      preload: join(__dirname, '../preload/index.js'),
      sandbox: false
    }
  })

  ConfigManager.init(FilePath, win)
  win.on('ready-to-show', async () => {
    initUpdateConfig()
    await initAppScript()

    // 显示主窗口
    win.show()
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
  // 创建托盘图标
  tray = new Tray(trayIcon) // 托盘图标路径

  // 创建托盘右键菜单
  const contextMenu = Menu.buildFromTemplate([
    {
      label: '显示主窗口',
      click: () => {
        win.setSkipTaskbar(false)
        win.show()
      }
    },
    {
      label: '退出',
      click: async () => {
        await sendCommand({
          type: 'exit'
        })
        startUltraVNC(`taskkill /IM winvnc.exe /F`)
        runShutdown()
        win.close()

        app.quit()
      }
    }
  ])

  tray.setToolTip('珂朵莉世界第一可爱...')
  tray.setContextMenu(contextMenu)

  // 双击托盘图标显示主窗口
  tray.on('double-click', () => {
    win.setSkipTaskbar(false)
    win.show()
  })

  ipcMain.on('min', () => win.minimize())
  ipcMain.on('max', () => win.maximize())
  ipcMain.on('unmaximize', () => win.unmaximize())
  ipcMain.on('close', (event) => {
    if (!app.isQuiting) {
      event.preventDefault()
      win.hide()
    }
  })

  ipcMain.on('openPath', () => {
    // 打开文件夹
    let scriptFile = join(FilePath, './script')
    shell.openPath(scriptFile)
  })

  ipcMain.handle('command', async (e, data) => {
    let list = JSON.parse(data[1])
    return handleData(data, list)
  })

  // 捕获未捕获的异常
  process.on('uncaughtException', (err) => {
    win.webContents.openDevTools({ mode: 'detach' })
    win.webContents.send('tips', `error: 主进程未捕获异常${err.message}`)
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
