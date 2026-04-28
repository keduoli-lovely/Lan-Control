import iconv from 'iconv-lite'
import { execSync } from 'child_process'

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

// 获取系统终端编码
const getCodePage = () => {
  try {
    const output = execSync('chcp', { encoding: 'utf8' })
    const match = output.match(/:\s*(\d+)/)
    return match ? parseInt(match[1], 10) : null
  } catch (error) {
    printLog(`[ReadJsonFile.js]  "${file}" 失败: 获取终端编码失败: ${error.message}`)
    return null
  }
}

export { printLog }
