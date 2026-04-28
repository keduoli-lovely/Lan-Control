import si from 'systeminformation'
import os from 'os'
import { AddCloseTask } from './initApp'
import { sendCommand, startReader } from '@keduoli-q/hardware-monitor'
import ConfigManager from './Config'

let win = null
let timer = null
let info = {}

function extractNumericValues(data) {
  const result = {}

  for (const key in data) {
    const value = data[key]
    // 匹配括号前的数字（支持负数和小数）
    const match = typeof value === 'string' ? value.match(/^([+-]?\d+(\.\d+)?)/) : null
    result[key] = match ? parseFloat(match[1]) : 0
  }

  return result
}

const initFile = async () => {
  const _ip = getLocalIP().address

  info.platform = os.version()
  ConfigManager.set('Config.ip', _ip)
  info.ip = _ip
  // 获取系统信息
  await getSystemInfo()

  if (info.gpu.length > 0) {
    win.webContents.send('tips', `${getCurrentTimeString()} GPU: ${info.gpu.join(', ')}`)
  } else {
    win.webContents.send('tips', `${getCurrentTimeString()} No GPU detected. Starting polling...`)
    await waitForRealGPU()
  }

  AddCloseTask(() => {
    clearInterval(timer)
  })
  await sendCommand({
    type: 'config',
    data: {
      intervalms: 5000
    }
  })
  await sendCommand({
    type: 'start',
    data: [
      'CPU Package',
      'Core Max',
      'CPU Total',
      'Memory',
      'GPU Package',
      'GPU Hot Spot',
      'D3D 3D'
    ]
  })
}

const getCurrentTimeString = () => {
  const now = new Date()
  const hours = String(now.getHours()).padStart(2, '0')
  const minutes = String(now.getMinutes()).padStart(2, '0')
  const seconds = String(now.getSeconds()).padStart(2, '0')
  return `[${hours}:${minutes}:${seconds}]`
}

async function getSystemInfo() {
  const cpuData = await si.cpu()
  const memData = await si.mem()
  const gpuData = await si.graphics()

  // 只保留真实显卡型号
  const realGPUs = gpuData.controllers
    .map((g) => g.model)
    .filter((model) => /NVIDIA|AMD/i.test(model))

  info.cpu = cpuData.brand
  info.memory = `${(memData.total / 1024 ** 3).toFixed(2)} GB`
  info.gpu = realGPUs
}

async function waitForRealGPU(interval = 10000) {
  let timer = null

  const checkGPU = async () => {
    const { gpu } = await getSystemInfo()
    if (gpu.length > 0) {
      clearInterval(timer)
      console.log('✅ Real GPU Detected:', gpu.join(', '))
    } else {
      console.log('⏳ No real GPU detected, retrying...')
    }
  }

  // Initial check
  await checkGPU()

  // Start polling if no real GPU found
  timer = setInterval(checkGPU, interval)
}

// 获取本地 192.168 开头的 IP 地址
const getLocalIP = () => {
  const interfaces = os.networkInterfaces()

  // 先找 192.168 开头的
  for (const name of Object.keys(interfaces)) {
    for (const iface of interfaces[name]) {
      if (iface.family === 'IPv4' && !iface.internal && iface.address.startsWith('192.168')) {
        return { name, address: iface.address }
      }
    }
  }

  // 如果没有找到，再返回第一个非内网 IPv4
  for (const name of Object.keys(interfaces)) {
    for (const iface of interfaces[name]) {
      if (iface.family === 'IPv4' && !iface.internal) {
        return { name, address: iface.address }
      }
    }
  }

  // 如果都没有，返回 null
  return null
}

const GetDefaultConfig = () => {
  win = ConfigManager.get('win')
  const AppConfig = ConfigManager.get('Config')

  info.nikename = AppConfig?.nikename || `keduoli_${new Date().getTime()}`
  const defaultConfig = {
    nikename: AppConfig?.nikename || `keduoli_${new Date().getTime()}`,
    AdPart: AppConfig?.AdPart || 41234,
    wspart: AppConfig?.wspart || 9191,
    defAdPart: AppConfig?.defAdPart || 41234,
    defwspart: AppConfig?.defwspart || 9191,
    autorun: AppConfig?.autorun || false
  }

  win.webContents.send('defaultConfig', defaultConfig)
}

const get = async () => {
  await startReader((err, data) => {
    if (err) {
      win.webContents.send('tips', `${getCurrentTimeString()} ${error.message}`)
    } else {
      info.nikename = ConfigManager.get('Config')?.nikename || info.nikename
      win.webContents.send('sensorsData', extractNumericValues(data))
      win.webContents.send('Systeminfo', info)
    }
  }, false)
}
get()

export { initFile, getCurrentTimeString, GetDefaultConfig, getLocalIP }
