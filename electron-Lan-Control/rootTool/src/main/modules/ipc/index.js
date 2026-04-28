import { ipcMain } from 'electron'
import { join } from 'path'
import { sendDataToPage } from '../../lifecycle'
import { printLog, sendToRenderer } from '../logger'
import { encode, decode } from '../../utils/crypto'
import { startUltraVNC } from '../vnc/startVnc'
import { getClient } from '../websocket/clients'
import { sendCommand, sendCommandFn } from '../websocket/sendCommand'
import { CheckFile } from '../../utils/CheckFile'
import { getFilePath, getAppConfig } from '../config/getConfig'
import { generateVncPassword } from '../vnc/password'
import { writeFileSync } from 'fs'
import { checkService } from '../script'

// 静态数据
let FilePath = null
let password = null
const images = new Map() // 存储图片base64数据的map

// 初始化静态数据
const initStaticData = () => {
  try {
    FilePath = getFilePath()
    password = getAppConfig().LinkPwd || 'keduoli'

    return { success: true }
  } catch (error) {
    return { success: false, error: `[initStaticData] ${error.message}` }
  }
}

// 远程查看被控设备
ipcMain.handle('remoteView', async (event, ip) => {
  if (FilePath != '' && CheckFile(FilePath)) {
    if (ip) {
      // UltraVNC路径
      sendCommand(getClient(ip).ws, 'verifyPassword', getAppConfig().password)
      const UltraVNCPath = join(FilePath, './lib/ultravnc_x64/vncviewer.exe')
      return await startUltraVNC(ip, UltraVNCPath, password || 'keduoli')
    }
  }
})

// 获取配置
ipcMain.handle('GetConfig', async () => {
  let App = getAppConfig()
  // 检测文件服务器状态checkService
  App['HttpSta'] = await checkService()
  return App
})

// 修改配置文件
ipcMain.handle('saveNewConfig', async (event, config) => {
  try {
    config = JSON.parse(config)
    if (!config) return
    const configPath = join(FilePath, 'config.json')
    let AppConfig = getAppConfig()
    for (let key in config) {
      AppConfig[key] = config[key]
    }

    if ('LinkPwd' in config) {
      let res = await generateVncPassword(config.LinkPwd, FilePath)
      if (res.sta) {
        AppConfig.password = res.data
        AppConfig.LinkPwd = config.LinkPwd
      }
    }

    AppConfig.LinkPwd = encode(AppConfig.LinkPwd)
    writeFileSync(configPath, JSON.stringify(AppConfig, null, 2))
    AppConfig.LinkPwd = decode(AppConfig.LinkPwd)
    return {
      sta: true,
      text: '配置文件修改完毕, 重启应用生效.',
      data: AppConfig
    }
  } catch (error) {
    return {
      sta: false,
      text: `配置文件修改失败, ${error.message}`
    }
  }
})

// 发送命令到被控端
ipcMain.handle('command', async (event, command) => {
  return await sendCommandFn(command)
})

// 处理接受到的被控端数据
const handleClientData = async (ip, data) => {
  // 处理被控端昵称
  if (data.type === 'nickname') {
    if (getClient(ip)) {
      getClient(ip).nikename = data?.nickname || `keduoli_${new Date().getTime()}`
    }
  }

  // 处理心跳
  if (data.type === 'heartbeat') {
    getClient(ip).lastSeen = Date.now()
    if (getClient(ip)) {
      getClient(ip).syncSta = data?.syncSta || false
    }
  }

  // 处理截图
  if (data.type === 'screenshot') {
    // 可以将图像渲染到 UI 或存储
    if (data.data) {
      images.set(ip, {
        pic: data.data
      })
    }

    // 发送当截图
    sendDataToPage('DesktopJpg')
  }

  // 处理命令执行结果
  if (data.type === 'script') {
    printLog(`[createWs.js]脚本执行结果:`, JSON.stringify(data))
    // 将数据发送到页面
    sendToRenderer('scriptResult', data)
  }

  // 脚本列表
  if (data.type === 'info') {
    let scriptLength = data?.configInfo || {}
    console.log(JSON.stringify(data), Object.keys(scriptLength).length ? data.configInfo : {})
    sendToRenderer('ScriptList', Object.keys(scriptLength).length ? data.configInfo : {})
  }

  // 同步脚本结果
  if (data.type === 'syncScript') {
    sendToRenderer('scriptResult', {
      ...data.data,
      duration: 5000
    })
  }
}

// 页面需要的数据格式
const formatDesktopJpgData = () => {
  const result = {}
  for (const [ip, value] of images.entries()) {
    result[ip] = value.pic
  }
  return result
}

export { initStaticData, handleClientData, formatDesktopJpgData }
