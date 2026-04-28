import { join } from 'path'
import { writeFileSync, existsSync } from 'fs'
import { decode } from '../../utils/crypto'
import { sendToRenderer } from '../logger'
import { requireJsonFile } from '../config'

let win = null
let FilePath = null
let AppConfig = {}
let configPath = null
let scriptList = null
let initialized = false

const initContext = (window, srcFilePath) => {
  if (initialized) return
  initialized = true

  try {
    configPath = join(srcFilePath, 'config.json')
    checkAndInitConfig(configPath)
    scriptList = requireJsonFile(join(srcFilePath, './data/config.json')) || { files: [] }
    win = window
    FilePath = srcFilePath
  } catch (error) {
    sendToRenderer('scriptResult', {
      sta: false,
      text: `初始化上下文失败: ${error.message}`,
      title: '初始化失败'
    })
  }
}

// 初始化应用配置
const checkAndInitConfig = (configPath) => {
  if (!existsSync(configPath)) {
    let defaultConfig = {
      AdPart: 41234,
      ADRunTime: 3000,
      wspart: 9191,
      password: '66C46F42995462527C',
      LinkPwd: '6b6564756f6c69',
      BROADCAST_INTERVAL: 5000,
      ADKey: 'keduoli_love',
      autorun: false,
      HttpPort: 10240
    }

    writeFileSync(configPath, JSON.stringify(defaultConfig, null, 2))
    defaultConfig.LinkPwd = decode(defaultConfig.LinkPwd)
    AppConfig = defaultConfig
  } else {
    AppConfig = requireJsonFile(configPath) || {}
    AppConfig.LinkPwd = decode(AppConfig.LinkPwd)
  }
}

// 返回路径
const getFilePath = () => {
  // 校验确保不为空
  if (!FilePath) {
    throw new Error('FilePath 未初始化，请先调用 initContext 方法')
  }
  return FilePath
}

// 获取应用配置
const getAppConfig = () => {
  return { ...AppConfig }
}

// 获取窗口
const getWindow = () => {
  // 校验确保不为空
  if (!win) {
    throw new Error('Window 未初始化，请先调用 initContext 方法')
  }
  return win
}

// 设置win
const setWindow = (_win) => {
  win = _win
}

// 设置应用配置
const setAppConfig = (newConfig) => {
  AppConfig = { ...AppConfig, ...newConfig }
}

// 获取http文件数据
const getScriptList = () => {
  return {
    ...scriptList
  }
}

export { initContext, getFilePath, getAppConfig, getWindow, setAppConfig, getScriptList, setWindow }
