import WebSocket from 'ws'
import { startScreenshotLoop, stopScreenshotLoop } from './screenDesktop'
import { handleCommand } from './RunScript'
import { AddCloseTask } from './initApp'
import { getCurrentTimeString } from './getSystemInfo'
import ConfigManager from './Config'

let win = null
let ws = null // WebSocket服务器实例
// WebSocket服务器端口
let SERVER_URL = 'ws://192.168.110.122:9191' // 控制端地址
// 被控端昵称(如果不存在则使用keduoli_拼接时间戳)
let NICKNAME = `keduoli_${new Date().getTime()}`
// 初始化WebSocket端口
const initWebSocket = (server) => {
  let name = ConfigManager.get('Config').nikename || `keduoli_${new Date().getTime()}`
  if (server) {
    SERVER_URL = server
  }
  if (name) {
    NICKNAME = name
  }

  win = ConfigManager.get('win')

  AddCloseTask(() => {
    if (ws) {
      ws.close()
    }
  })
}

// 启动WebSocket连接
const connect = () => {
  ws = new WebSocket(SERVER_URL)

  ws.on('open', () => {
    win.webContents.send('tips', `${getCurrentTimeString()} 已连接到控制端`)

    startScreenshotLoop(2000) // 启动截图循环
    // 发送被控端的昵称
    sendMessage({ type: 'nickname', nickname: NICKNAME })

    // 启动心跳定时器
    setInterval(() => {
      sendMessage({
        type: 'heartbeat',
        timestamp: Date.now(),
        syncSta: ConfigManager.get('syncSta')
      })
    }, 5000)
  })

  ws.on('message', (message) => {
    try {
      const data = JSON.parse(message)

      if (data.type === 'command') {
        handleCommand(data.payload, data?.nikename)
      }
    } catch (err) {
      win.webContents.send('tips', `${getCurrentTimeString()} 消息解析失败: ${err}`)
    }
  })

  ws.on('close', () => {
    stopScreenshotLoop() // 停止截图循环
    win.webContents.send('tips', `${getCurrentTimeString()} 连接关闭，尝试重连...`)
    setTimeout(connect, 3000)
  })

  ws.on('error', (err) => {
    stopScreenshotLoop() // 停止截图循环
    win.webContents.send('tips', `${getCurrentTimeString()} 连接错误: ${err}`)
  })
}

const sendMessage = (obj) => {
  if (ws && ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify(obj))
  }
}

export { initWebSocket, connect, sendMessage }
