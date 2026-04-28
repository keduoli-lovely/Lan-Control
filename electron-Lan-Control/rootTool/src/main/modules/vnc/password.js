import { join } from 'path'
import { spawn } from 'child_process'
import { existsSync, readFileSync } from 'fs'
import { sendToRenderer } from '../logger'

/**
 * 使用 UltraVNC 的 createpassword 工具生成加密密码
 * @param {string} password - 明文密码
 * @param {string} workingDir - createpassword.exe 执行目录（会在此生成 ultravnc.ini）
 * @returns {Promise<string>} 加密后的 passwd 字段
 */
const generateVncPassword = (password, FilePath) => {
  return new Promise((resolve) => {
    const workingDir = join(FilePath, 'lib', 'ultravnc_x64')
    const exePath = join(workingDir, 'createpassword.exe')
    const child = spawn(exePath, [password], {
      cwd: workingDir,
      windowsHide: true
    })

    child.on('error', (err) => {
      sendToRenderer('scriptResult', {
        sta: false,
        text: `执行失败: ${err.message}`,
        title: '密码修改失败'
      })

      resolve({ sta: false })
    })
    child.on('exit', (code) => {
      const iniPath = join(workingDir, 'ultravnc.ini')
      if (!existsSync(iniPath)) {
        sendToRenderer('scriptResult', {
          sta: false,
          text: '未生成 ultravnc.ini',
          title: '密码修改失败'
        })
        return resolve({ sta: false })
      }
      const content = readFileSync(iniPath, 'utf-8')
      const parsed = require('ini').parse(content)
      const encrypted = parsed.UltraVNC?.passwd
      if (!encrypted) {
        sendToRenderer('scriptResult', {
          sta: false,
          text: '未找到 passwd 字段',
          title: '密码修改失败'
        })

        resolve({ sta: false })
      }

      resolve({ sta: true, data: encrypted })
    })
  })
}

export { generateVncPassword }
