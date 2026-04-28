const screenshot = require('screenshot-desktop')
import { printLog } from './log'
import { sendMessage } from './GetWs'
import ConfigManager from './Config'

// 定时器
let screenshotTimer = null

// 截图并将图片转为base64通过ws发送页面
const takeScreenshot = async () => {
  try {
    const img = await screenshot({ format: 'jpg' })
    const base64Image = img.toString('base64')
    return `data:image/jpeg;base64,${base64Image}`
  } catch (err) {
    printLog('[screenshot.js] 截图失败:', err)
    let win = ConfigManager.get('win')
    win.webContents.send('tips', `${getCurrentTimeString()} ${error.message}`)
    return null
  }
}

const startScreenshotLoop = (interval = 2000) => {
  if (screenshotTimer) return

  screenshotTimer = setInterval(async () => {
    const imgData = await takeScreenshot()
    if (imgData) {
      printLog('[screenshot-loop] 截图成功，发送数据')
      sendMessage({
        type: 'screenshot',
        data: imgData
      })
    }
  }, interval)

  printLog('[screenshot-loop] 已启动截图定时器')
}

const stopScreenshotLoop = () => {
  if (screenshotTimer) {
    clearInterval(screenshotTimer)
    screenshotTimer = null
    printLog('[screenshot-loop] 已停止截图定时器')
  }
}

export { startScreenshotLoop, stopScreenshotLoop }
