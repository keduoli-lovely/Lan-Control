import { existsSync } from 'fs'
import { printLog } from '../modules/logger'

// 检查文件是否存在并返回布尔值
const CheckFile = (file) => {
  if (!existsSync(file)) {
    printLog(`[checkFile] 文件路径不存在: ${file}`)
    return false
  } else {
    return true
  }
}
export { CheckFile }
