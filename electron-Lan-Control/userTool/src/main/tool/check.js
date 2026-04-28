import { existsSync } from 'fs'
import { createVncIniFile } from './ReadJsonFile'
import { startUltraVNC } from './RunScript'
import { getCurrentTimeString } from './getSystemInfo'
import ConfigManager from './Config'

let AppConfig = null
let configPath = null
let win = null
let UltraVNCPath = null
let UltraVNCConfig = null

// check.js 初始化
const initCheck = () => {
  AppConfig = ConfigManager.get('Config')
  configPath = ConfigManager.get('configPath')
  win = ConfigManager.get('win')
  UltraVNCPath = ConfigManager.get('UltraVNCPath')
  UltraVNCConfig = ConfigManager.get('UltraVNCConfig')
}

// 在这里vnc应用程序初始化脚本
const checkVncFile = async () => {
  if (existsSync(UltraVNCPath)) {
    if (!existsSync(UltraVNCConfig)) {
      await createVncIniFile(UltraVNCConfig, AppConfig.password)
    }
    try {
      await startUltraVNC(`taskkill /IM winvnc.exe /F`)
    } catch (error) {}

    const command = `"${UltraVNCPath}" -run -config "${UltraVNCConfig}"`
    startUltraVNC(command)
  } else {
    win.webContents.send('tips', `${getCurrentTimeString()} 未找到 UltraVNC 程序，请检查！`)
  }
}

export { initCheck, checkConfigFile, checkVncFile }
