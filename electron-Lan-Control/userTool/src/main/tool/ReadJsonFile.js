import { writeFileSync } from "fs"
import { printLog } from "./log"

function requireJsonFile(file) {
  try {
    return require(file) || {}
  } catch (err) {
    printLog(`[Error] 读取文件 "${file}" 失败: ${err.message} --SaveJsonFile.js`)
    return null
  }
}

const createVncIniFile = (filePath, passwd) => {
  const content = `
      [UltraVNC]
      passwd=${passwd}
      DisableTrayIcon=1
  `
  writeFileSync(filePath, content)
}

export { requireJsonFile, createVncIniFile }
