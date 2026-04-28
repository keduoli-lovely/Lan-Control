import iconv from 'iconv-lite'
import { getCodePage } from '../config'
import { getWindow } from '../config/getConfig'

const printLog = (message, over) => {
  const encoding = getCodePage() === 65001 ? 'utf8' : 'gbk'
  const parts = [iconv.encode(message, encoding)]

  if (over !== undefined && over !== null && over !== '') {
    parts.push(iconv.encode(' ', encoding))
    parts.push(iconv.encode(String(over), encoding))
  }

  parts.push(iconv.encode('\n', encoding))

  process.stdout.write(Buffer.concat(parts))
}

// 发送消息到渲染进程
const sendToRenderer = (channel, payload) => {
  const win = getWindow()
  if (win?.webContents) {
    win.webContents.send(channel, payload)
  } else {
    console.warn(`[context] 无法发送消息，窗口未初始化: ${channel}`)
  }
}

export { printLog, sendToRenderer }
