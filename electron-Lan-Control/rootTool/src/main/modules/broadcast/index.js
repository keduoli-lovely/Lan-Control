import dgram from 'dgram' // 引入 Node.js 的 UDP 模块
import { printLog, sendToRenderer } from '../logger'
import { CloseRunTask } from '../../lifecycle'
import { getLocalIP } from '../../utils/getLocalIPs'
import { getAppConfig } from '../config/getConfig'

let BROADCAST_PORT = 41234 // 设置广播端口
let BROADCAST_INTERVAL = 3000 // 每隔 3 秒广播一次
let BROADCAST_WSPART = 9191 // WebSocket 端口
let BROADCAST_KEY = 'keduoli_love' // 广播key
let udpSocket = null // UDP socket
let HTTPSTA = 10240 // http 服务端口

// 初始化覆盖广播端口和触发时间
const initBroadcast = () => {
  try {
    BROADCAST_PORT = getAppConfig().AdPart || 41234
    BROADCAST_INTERVAL = getAppConfig().ADRunTime || 3000
    BROADCAST_WSPART = getAppConfig().wspart || 9191
    BROADCAST_KEY = getAppConfig().ADKey || 'keduoli_love'
    HTTPSTA = getAppConfig().HttpPort || 10240

    return { success: true }
  } catch (error) {
    return { success: false, error: `[initBroadcast] ${error.message}` }
  }
}

const startBroadcast = () => {
  const broadcastIP = getValidBroadcastIP()
  if (!broadcastIP) {
    sendToRenderer('scriptResult', {
      sta: false,
      text: '未获取到有效的广播 IP 地址'
    })
    return
  }

  setInterval(() => {
    const message = Buffer.from(`${BROADCAST_KEY} ${BROADCAST_WSPART} ${HTTPSTA}`) // 创建要广播的消息
    udpSocket.send(message, BROADCAST_PORT, broadcastIP, (err) => {
      if (err) {
        printLog('[SendAD.js]广播消息发送失败:', err)
        sendToRenderer('scriptResult', {
          sta: false,
          text: `广播消息发送失败: ${err.message}`,
          title: '广播失败'
        })
      } else {
        printLog('[SendAD.js]广播消息已发送:', message.toString())
      }
    })
  }, BROADCAST_INTERVAL)

  // 关闭广播服务
  CloseRunTask(() => {
    udpSocket.close()
  })
}

// 获取广播地址
const getValidBroadcastIP = () => {
  const localIPs = getLocalIP()
  if (!localIPs.length) {
    sendToRenderer('scriptResult', {
      sta: false,
      text: '未获取到有效的本地 IP 地址',
      title: '获取IP失败'
    })
    return null
  }

  const ip = localIPs[0].address
  const match = ip.match(/^(\d+\.\d+\.\d+)\.\d+$/)
  if (!match) {
    sendToRenderer('scriptResult', {
      sta: false,
      text: '未获取到有效的广播 IP 地址',
      title: '获取IP失败'
    })
    return null
  }

  return `${match[1]}.255`
}

// 使能局域网广播功能
const enableLanBroadcast = () => {
  if (udpSocket) {
    return { success: true }
  }

  udpSocket = dgram.createSocket('udp4') // 创建一个 UDP socket，使用 IPv4

  try {
    udpSocket.bind(9191, () => {
      udpSocket.setBroadcast(true)
      startBroadcast()
      printLog('UDP 广播已启用')
    })

    return { success: true }
  } catch (err) {
    printLog('UDP 绑定失败:', err)
    sendToRenderer('scriptResult', {
      sta: false,
      text: `UDP 绑定失败: ${err.message}`,
      title: '广播失败'
    })
    return { success: false, error: `[enableLanBroadcast] ${err.message}` }
  }
}

export { initBroadcast, enableLanBroadcast }
