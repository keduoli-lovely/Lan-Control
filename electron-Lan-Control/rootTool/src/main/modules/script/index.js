import { shell, dialog } from 'electron'
import { exec } from 'child_process'
import axios from 'axios'
import { getAppConfig } from '../config/getConfig'
import { sendToRenderer } from '../logger'

let TryNum = 0

// 打开网址
const openUrl = (link) => {
  shell.openExternal(link)
}

// 选择文件夹
async function selectFileOrFolder(flag) {
  const properties = flag ? ['openFile'] : ['openDirectory']

  return dialog.showOpenDialog({ properties }).then((result) => {
    if (!result.canceled && result.filePaths.length > 0) {
      return { sta: true, path: result.filePaths[0], text: '文件选择完毕' }
    } else {
      return { sta: false, path: '', text: '文件选择失败请重新选择' }
    }
  })
}

// 复制文件到http文件夹

/**
 * 执行命令行指令
 * @param {string} command 要执行的命令
 * @returns {Promise<{ sta: boolean, stdout: string, stderr: string }>}
 */
function runCommand(command) {
  return new Promise((resolve) => {
    exec(command, { encoding: 'utf8' }, (error, stdout, stderr) => {
      if (error) {
        resolve({ sta: false, text: error })
      } else {
        resolve({ sta: true, text: '文件选择完毕' })
      }
    })
  })
}

// 返回文件服务器状态
async function checkService() {
  try {
    // 用 HEAD 请求更轻量，只判断资源是否可访问
    let url = getAppConfig()?.HttpPort || 10240
    const res = await axios.head(`http://localhost:${url}/config`, {
      timeout: 2000, // 避免长时间挂起
      validateStatus: (status) => status >= 200 && status < 400 // 只要不是错误就算成功
    })
    return true
  } catch (err) {
    if (TryNum > 5) {
      TryNum = 0
      sendToRenderer('scriptResult', {
        sta: false,
        text: `访问次数过多，停止检查: ${err}`,
        title: '服务检查失败'
      })

      return false
    }
    TryNum += 1
    sendToRenderer('scriptResult', {
      sta: false,
      text: `文件服务未启动或不可访问: ${err.message} 第 ${TryNum} 次重试中...`,
      title: '服务检查失败'
    })
    setTimeout(() => {
      checkService()
    }, 4000)
  }
}

export { openUrl, selectFileOrFolder, runCommand, checkService }
