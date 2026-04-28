import WebSocket from 'ws'
import { printLog, sendToRenderer } from '../logger'
import { CloseRunTask, sendDataToPage } from '../../lifecycle'
import { handleClientData } from '../ipc'
import { sendCommand } from './sendCommand'
import { registerClient, getClient, removeClient, getAllClients, getClientCount } from './clients'
import { getAppConfig } from '../config/getConfig'

let PORT = 8080
let wss = null // WebSocket服务器实例
let heartbeatInterval = null // 心跳定时器
let heartbeatTimeout = 10000 // 心跳频率
let password = '66C46F42995462527C' // 默认密码

// 初始化WebSocket服务器端口
const initWebSocket = () => {
  PORT = getAppConfig().wspart || PORT
  password = getAppConfig().password || password
  heartbeatTimeout = getAppConfig().BROADCAST_INTERVAL || heartbeatTimeout

  try {
    wss = new WebSocket.Server({ port: PORT })

    // 关闭WebSocket服务器
    CloseRunTask(() => {
      if (wss) {
        wss.close()
      }
      if (heartbeatInterval) {
        clearInterval(heartbeatInterval)
      }
    })

    return { success: true }
  } catch (error) {
    if (wss && wss.address()) {
      wss.close()
    }
    sendToRenderer('scriptResult', {
      sta: false,
      text: `WebSocket启动失败: ${err.message}`,
      title: 'WebSocket启动失败'
    })
    return { success: false, error: `[initWebSocket] ${error.message}` }
  }
}

// 启动WebSocket服务器
const startWebSocket = () => {
  if (!wss) {
    sendToRenderer('scriptResult', {
      sta: false,
      text: '[startWebSocket] WebSocket服务器未初始化',
      title: 'WebSocket启动失败'
    })
    return { success: false, error: '[startWebSocket] WebSocket服务器未初始化' }
  }

  try {
    wss.on('connection', (ws, req) => {
      let ip = req.socket.remoteAddress
      if (ip.startsWith('::ffff:')) {
        ip = ip.replace('::ffff:', '')
      }

      printLog(`[createWs.js]被控端连接:`, ip)
      registerClient(ip, ws)
      sendDataToPage('newClient')
      printLog(`[createWs.js]设备 ${ip} 连接成功`)
      sendCommand(getClient(ip).ws, 'verifyPassword', password)

      ws.on('message', (message) => {
        try {
          const data = JSON.parse(message)
          printLog(`[createWs.js]收到来自 ${ip} 的消息:`, data)
          handleClientData(ip, data)
        } catch (err) {
          sendToRenderer('scriptResult', {
            sta: false,
            text: `[createWs.js]消息解析失败: ${err.message}`,
            title: 'WebSocket消息解析失败'
          })
        }
      })

      ws.on('close', () => {
        removeClient(ip)
        sendDataToPage('newClient')

        sendToRenderer('scriptResult', {
          sta: false,
          text: `[createWs.js]${getClient(ip)?.nikename || ip}断开连接`,
          title: '连接断开'
        })
      })

      ws.on('error', (err) => {
        removeClient(ip)
        sendDataToPage('newClient')
        sendToRenderer('scriptResult', {
          sta: false,
          text: `[createWs.js]连接错误: ${err.message}`,
          title: 'WebSocket连接错误'
        })
      })
    })

    printLog(`[createWs.js]控制端 WebSocket 服务已启动，监听端口 ${PORT}`)
    return { success: true }
  } catch (err) {
    sendToRenderer('scriptResult', {
      sta: false,
      text: `[startWebSocket] 注册连接事件失败: ${err.message}`,
      title: 'WebSocket启动失败'
    })
    return { success: false, error: `[startWebSocket] ${err.message}` }
  }
}

// 心跳检测
heartbeatInterval = setInterval(() => {
  const now = Date.now()
  for (const [ip, info] of getAllClients()) {
    if (now - info.lastSeen > 10000) {
      info.ws.terminate()
      removeClient(ip)

      sendToRenderer('scriptResult', {
        sta: false,
        text: `[createWs.js]设备 ${ip} 心跳超时`,
        title: '设备超时'
      })
    } else {
      printLog(
        `[createWs.js]设备 ${ip} 在线，最后心跳时间: ${new Date(info.lastSeen).toLocaleTimeString()}`
      )
    }
  }

  // 发送当前在线设备列表
  if (getClientCount()) {
    sendDataToPage('newClient')
  }
}, heartbeatTimeout)

export { initWebSocket, startWebSocket }
