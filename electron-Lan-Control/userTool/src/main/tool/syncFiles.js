import fs from 'fs'
import path from 'path'
import axios from 'axios'
import { requireJsonFile } from './ReadJsonFile'
import { runCommand } from './RunScript'
import { getCurrentTimeString, getLocalIP } from './getSystemInfo'
import { sendMessage } from './GetWs'
import ConfigManager from './Config'

let SERVER_URL = 'http://127.0.0.1:10240' // 发送端地址
let DOWNLOAD_DIR = null
let DONWLOAD_CONFIG = null
let configList = null
let AppConfigPath = null
let LIBDIR = null
let TryNum = 0
let win = null
let isUpdate = false
let _ip = null
let SCRIPT = null

// 确保下载目录存在
const InitFile = () => {
  const file = ConfigManager.get('FilePath')
  win = ConfigManager.get('win')
  AppConfigPath = ConfigManager.get('configPath')

  DOWNLOAD_DIR = path.join(file, './data')
  LIBDIR = path.join(file, './lib')
  SCRIPT = path.join(file, './script')
  DONWLOAD_CONFIG = path.join(DOWNLOAD_DIR, './config.json')
  if (!fs.existsSync(DOWNLOAD_DIR)) {
    fs.mkdirSync(DOWNLOAD_DIR)
  }
  if (!fs.existsSync(DONWLOAD_CONFIG)) {
    fs.writeFileSync(
      DONWLOAD_CONFIG,
      JSON.stringify(
        {
          files: []
        },
        null,
        2
      )
    )

    configList = { files: [] }
  } else {
    configList = requireJsonFile(DONWLOAD_CONFIG) || { files: [] }
  }
}

async function downloadFile(url, outputPath) {
  const writer = fs.createWriteStream(outputPath)

  try {
    const response = await axios({
      method: 'get',
      url,
      responseType: 'stream'
    })

    response.data.pipe(writer)

    return new Promise((resolve, reject) => {
      writer.on('finish', () => {
        resolve(outputPath)
      })
      writer.on('error', (err) => {
        win.webContents.send('tips', `${getCurrentTimeString()} ${error.message}`)
        reject(err)
      })
    })
  } catch (error) {
    win.webContents.send('tips', `${getCurrentTimeString()} ${error.message}`)
    throw error
  }
}

function getDifferentFilenames(data1, data2) {
  const filenames1 = new Set(data1.files.map((f) => f.filename))
  const filenames2 = new Set(data2.files.map((f) => f.filename))

  const diff1 = data1.files.filter((f) => !filenames2.has(f.filename))
  const diff2 = data2.files.filter((f) => !filenames1.has(f.filename))

  return [...diff1, ...diff2]
}

async function syncFiles() {
  isUpdate = true
  let result = null
  const zip7 = path.join(LIBDIR, './7zip/7za.exe')
  const configUrl = `${SERVER_URL}/config`

  try {
    const { data } = await axios.get(configUrl)

    if (!data.files || !Array.isArray(data.files)) {
      ConfigManager.set('syncSta', false)
      win.webContents.send('tips', `${getCurrentTimeString()}  config.json 格式无效或为空`)
      isUpdate = false
      return
    }

    if (!data?.files?.length) {
      configList = data
      ConfigManager.set('syncSta', true)
      isUpdate = false
      return
    }
    result = configList?.files.length ? getDifferentFilenames(data, configList) : data.files
    if (result.length) {
      for (const item of result) {
        try {
          await downloadFile(`${SERVER_URL}${item.url}`, `${DOWNLOAD_DIR}\\${item.filename}`)
          configList.files.push(item)

          if (ConfigManager.get('Config')?.script) {
            if (item.fielOrDoc === 2) {
              const nameWithoutExt = path.basename(item.filename, path.extname(item.filename))
              runCommand(`${zip7} x ${DOWNLOAD_DIR}\\${item.filename} -o${SCRIPT}`)

              ConfigManager.set(`Config.script.${item?.code || item.filename}`, {
                name: item?.code || item.filename,
                path: path.join(SCRIPT, `${nameWithoutExt}${item?.code ? '\\' + item.code : ''}`)
              })
            } else if (item.fielOrDoc === 1) {
              ConfigManager.set(`Config.script.${item.filename}`, {
                name: item.filename,
                path: `${DOWNLOAD_DIR}\\${item.filename}`,
                code: item?.code || ''
              })
            }
          }
        } catch (err) {
          ConfigManager.set('syncSta', false)
          isUpdate = false
          win.webContents.send('tips', `${getCurrentTimeString()} ${err.message}`)
        }
      }

      fs.writeFileSync(DONWLOAD_CONFIG, JSON.stringify(configList, null, 2))
      fs.writeFileSync(AppConfigPath, JSON.stringify(ConfigManager.get('Config'), null, 2))
      ConfigManager.set('syncSta', true)
      isUpdate = false
      sendMessage({
        type: 'syncScript',
        data: {
          sta: true,
          text: `完成同步`,
          title: _ip
        }
      })
    } else {
      ConfigManager.set('syncSta', true)
      isUpdate = false
      win.webContents.send('tips', `${getCurrentTimeString()} 同步完成: 没有可以同步的脚本`)
    }
  } catch (err) {
    ConfigManager.set('syncSta', false)
    isUpdate = false
    win.webContents.send('tips', `${getCurrentTimeString()} ${err.message}`)
  }
}

async function checkService() {
  if (!ConfigManager.get('Config')?.LinkIp) return
  _ip = ConfigManager.get('Config')?.ip || getLocalIP().address

  SERVER_URL = `http://${ConfigManager.get('Config')?.LinkIp}:${ConfigManager.get('Config').httpPort}`
  try {
    // 用 HEAD 请求更轻量，只判断资源是否可访问
    const res = await axios.head(`${SERVER_URL}/config`, {
      timeout: 2000, // 避免长时间挂起
      validateStatus: (status) => status >= 200 && status < 400 // 只要不是错误就算成功
    })
    if (!isUpdate) await syncFiles()
    return {
      sta: true,
      text: `开始同步.....`,
      title: _ip
    }
  } catch (err) {
    if (TryNum > 4) {
      TryNum = 0
      win.webContents.send(
        'tips',
        `${getCurrentTimeString()}同步失败：尝试次数过多${err.message} ${SERVER_URL}`
      )

      return {
        sta: false,
        text: `同步失败：尝试次数超出限制 ${err.message}`,
        title: _ip
      }
    }
    TryNum += 1
    win.webContents.send(
      'tips',
      `${getCurrentTimeString()}同步失败：文件服务未启动或不可访问${err.message} 第 ${TryNum} 次重试中... ${SERVER_URL}`
    )

    setTimeout(() => checkService(), 4000)
    return {
      sta: false,
      text: `同步失败：文件服务未启动或不可访问${err.message}`,
      title: _ip
    }
  }
}

export { InitFile, checkService }
