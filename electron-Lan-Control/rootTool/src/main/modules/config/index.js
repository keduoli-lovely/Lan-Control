import { promises as fs } from 'fs'
import { printLog, sendToRenderer } from '../logger'
import { runCommand } from '../script'
import { execSync } from 'child_process'

const requireJsonFile = (file) => {
  try {
    return require(file) || {}
  } catch (err) {
    printLog(`[ReadJsonFile.js] 读取文件 "${file}" 失败: ${err.message}`)
    sendToRenderer('scriptResult', {
      sta: false,
      text: `读取文件失败: ${err.message}`,
      title: '读取失败'
    })
    return null
  }
}

// 复制文件到目录
const copyFile = async (file, toPath) => {
  try {
    await fs.copyFile(file, toPath)
    return {
      sta: true,
      text: '提交成功'
    }
  } catch (err) {
    sendToRenderer('scriptResult', {
      sta: false,
      text: err.message,
      title: '复制失败'
    })
    return {
      sta: false,
      text: err.message
    }
  }
}

// 压缩文件夹并复制
const tarFileZipToPath = async (file, toPath, zip7) => {
  let res = await runCommand(`${zip7} a -tzip -mx=1 ${toPath} ${file}`)
  sendToRenderer('scriptResult', {
    sta: res.sta,
    text: res.text,
    title: res.sta ? '压缩成功' : '压缩失败'
  })
  return res
}

// 获取系统终端编码
const getCodePage = () => {
  try {
    const output = execSync('chcp', { encoding: 'utf8' })
    const match = output.match(/:\s*(\d+)/)
    return match ? parseInt(match[1], 10) : null
  } catch (error) {
    sendToRenderer('scriptResult', {
      sta: false,
      text: `获取终端编码失败: ${error.message}`,
      title: '获取失败'
    })
    return null
  }
}

export { requireJsonFile, tarFileZipToPath, copyFile, getCodePage }
