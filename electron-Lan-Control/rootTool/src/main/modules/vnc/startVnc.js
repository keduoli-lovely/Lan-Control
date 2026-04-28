import { exec } from 'child_process'
import { printLog } from '../logger'

// 启动 UltraVNC 并返回 Promise
const startUltraVNC = (ip, filepath, password) => {
  const command = `"${filepath}" -connect ${ip} -password ${password}`
  printLog(`[UltraVNC] 启动命令: ${command}`)
  return new Promise((resolve) => {
    exec(command, (error, stdout, stderr) => {
      if (error) {
        printLog('[UltraVNC] 启动失败:', error.message)
        resolve([false, error.message])
        return
      }
      printLog('[UltraVNC] 启动成功')
      resolve([true])
    })
  })
}

export { startUltraVNC }
