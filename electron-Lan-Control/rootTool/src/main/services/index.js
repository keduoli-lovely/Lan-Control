import { join } from 'path'
import { writeFileSync, existsSync } from 'fs'
import express from 'express'
import { printLog } from '../modules/logger'
import { getFilePath, getAppConfig } from '../modules/config/getConfig'

const app = express()

// 启动http服务器
const runHttpServer = () => {
  try {
    let serverState = getFilePath()
    let serverPort = getAppConfig()?.HttpPort || 10240
    const fileDoc = join(serverState, './data')
    const config = join(fileDoc, './config.json')
    if (!existsSync(config)) {
      writeFileSync(config, JSON.stringify({ files: [] }, null, 2))
    }

    app.use('/files', express.static(fileDoc))

    app.get('/config', (req, res) => {
      res.sendFile(config)
    })

    app.listen(serverPort, '0.0.0.0', () => {
      printLog(`文件服务已启动：http://localhost:${serverPort}/files`)
    })

    return { success: true }
  } catch (error) {
    return { success: false, error: `[runHttpServer] ${error.message}` }
  }
}

export { runHttpServer }
