import { initBroadcast, enableLanBroadcast } from '../modules/broadcast'
import { initStaticData, formatDesktopJpgData } from '../modules/ipc'
import { printLog, sendToRenderer } from '../modules/logger'
import { formatNewClientData } from '../modules/websocket/clients'
import { initWebSocket, startWebSocket } from '../modules/websocket/index'
import { runHttpServer } from '../services'

let CloseRun = []
// 防止热重载输出ws报错
let hasRunInit = false

const runInit = async () => {
  if (hasRunInit) {
    printLog('[runInit] 已初始化，跳过重复调用')
    return
  }
  hasRunInit = true

  const steps = [
    { name: 'initBroadcast', fn: () => initBroadcast() }, // 初始化广播端口和间隔
    { name: 'initWebSocket', fn: () => initWebSocket() }, // 初始化WebSocket服务
    { name: 'enableLanBroadcast', fn: () => enableLanBroadcast() }, // 启动广播
    { name: 'startWebSocket', fn: () => startWebSocket() }, // 启动WebSocket服务器
    { name: 'initStaticData', fn: () => initStaticData() }, // 初始化静态数据
    { name: 'runHttpServer', fn: () => runHttpServer() } // 启动文件服务器
  ]

  for (const step of steps) {
    try {
      const result = await step.fn()
      if (!result.success) {
        printLog(`[runInit] 步骤失败: ${step.name}`)
        sendToRenderer('scriptResult', {
          sta: false,
          text: `初始化失败，步骤: ${step.name}，错误: ${result.error}`,
          title: '初始化失败'
        })
        if (result.error) printLog(`[runInit]`, result.error)
        return
      }
    } catch (err) {
      printLog(`[runInit] 异常中断: ${step.name}`, err)
      sendToRenderer('scriptResult', {
        sta: false,
        text: `初始化失败，步骤: ${step.name}，错误: ${err.message}`,
        title: '初始化失败'
      })
      return
    }
  }

  printLog('[runInit] 所有初始化步骤完成')
}

// 注册关闭时执行的任务
const CloseRunTask = (fn) => {
  CloseRun.push(fn)
}
// 执行关闭时的任务
const runShutdown = async () => {
  for (const fn of CloseRun) await fn()
}

// 发送数据到页面
const sendDataToPage = (type) => {
  const obj = { type }

  if (type === 'newClient') {
    obj.data = formatNewClientData()
    printLog('当前在线设备数:', obj.data)
  } else if (type === 'DesktopJpg') {
    obj.data = formatDesktopJpgData()
  }

  sendToRenderer('message', JSON.stringify(obj))
}

export { runInit, runShutdown, sendDataToPage, CloseRunTask }
