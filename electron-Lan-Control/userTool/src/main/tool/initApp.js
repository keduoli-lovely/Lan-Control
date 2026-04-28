import { app } from 'electron'
import { initGetPath } from './RunScript'
import { initFile, getCurrentTimeString, GetDefaultConfig } from './getSystemInfo'
import { initBroadcast, startListening } from './GetAD'
import { initWebSocket, connect } from './GetWs'
import { initCheck, checkVncFile } from './check'
import { InitFile, checkService } from './syncFiles'
import { printLog } from './log'
import ConfigManager from './Config'

let CloseRun = []

const AddCloseTask = (task) => {
  CloseRun.push(task)
}

const runShutdown = async () => {
  for (const task of CloseRun) {
    await task()
  }
}

const initAppScript = async () => {
  const win = ConfigManager.get('win')
  // 初始化文件
  initCheck()
  initBroadcast()
  initGetPath()
  // 在这里vnc应用程序初始化脚本
  checkVncFile()
  // 启动监听广播
  startListening((rinfo, msg, serverIp) => {
    win &&
      win.webContents.send(
        'tips',
        `${getCurrentTimeString()} Received broadcast from ${rinfo.address}:${rinfo.port}`
      )
    win &&
      win.webContents.send(
        'tips',
        `${getCurrentTimeString()} Server IP: ws://${rinfo.address}:${serverIp || ConfigManager.get('Config').wspart}`
      )
    initWebSocket(`ws://${rinfo.address}:${serverIp || ConfigManager.get('Config').wspart}`)
    // 启动WebSocket连接
    connect()
    // 同步控制端ws端口
    ConfigManager.set('Config.wspart', serverIp)
  })

  // 定时获取数据
  setTimeout(() => {
    // 校验配置文件config.json
    ConfigManager.checkConfigFile()
    initFile()
    // 脚本同步
    checkService()
  }, 5000)

  // 收集默认配置
  GetDefaultConfig()

  InitFile()

  // 检测开机自启动
  applyAutorunSetting(ConfigManager.get('Config').autorun)
}

/**
 * 根据配置开启或关闭自启动
 * @param {Object} AppConfig - 应用配置对象
 * @param {boolean} AppConfig.autorun - 是否开启自启动
 */
function applyAutorunSetting(runkey) {
  const settings = app.getLoginItemSettings()
  if (settings.openAtLogin === runkey) {
    printLog('传入一样无需修改', settings.openAtLogin,1, runkey)
    return
  }

  app.setLoginItemSettings({
    openAtLogin: runkey === true,
    openAsHidden: false, // macOS 可选，Windows忽略
    path: app.getPath('exe'), // 指定当前应用的可执行文件
    args: [] // 可选参数
  })
  printLog('修改完成')
}

export { AddCloseTask, runShutdown, initAppScript, applyAutorunSetting }
