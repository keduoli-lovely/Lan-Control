import dgram from 'dgram' // 引入 Node.js 的 UDP 模块
import { AddCloseTask } from './initApp'
import { getCurrentTimeString } from './getSystemInfo'
import ConfigManager from './Config'

// 初始化监听广播端口
let BROADCAST_PORT = 41234 // 设置默认广播端口
let win = null

// 初始化
const client = dgram.createSocket('udp4') // 创建一个 UDP socket，使用 IPv4
const initBroadcast = () => {
  BROADCAST_PORT = ConfigManager.get('Config').BROADCAST_PORT || 41234
  win = ConfigManager.get('win')
}

// 暴露一个方法用于启动监听广播
const startListening = (onBroadcast) => {
  const handler = (msg, rinfo) => {
    // 切割字符串
    let message = msg.toString().split(' ')
    if (message[0] === 'keduoli_love') {
      ConfigManager.set('Config.wspart', message[1])
      ConfigManager.set('Config.httpPort', message[2])
      ConfigManager.set('Config.LinkIp', rinfo.address)
      win.webContents.send(
        'tips',
        `${getCurrentTimeString()} Received broadcast from ${rinfo.address}:${rinfo.port}`
      )
      if (typeof onBroadcast === 'function') {
        onBroadcast(rinfo, msg, message[1])
      }

      // ✅ 在这里关闭监听器和客户端
      client.removeListener('message', handler)
      client.close(() => {
        win.webContents.send('tips', `${getCurrentTimeString()} UDP client manually closed.`)
      })
    }
  }

  client.on('message', handler)

  client.bind(BROADCAST_PORT, () => {
    win.webContents.send(
      'tips',
      `${getCurrentTimeString()} UDP client listening on port ${BROADCAST_PORT}`
    )
  })

  AddCloseTask(() => {
    try {
      client.close()
    } catch (error) {
      win.webContents.send('tips', `${getCurrentTimeString()} ${error.message}`)
    }
  })
}

export { initBroadcast, startListening }
