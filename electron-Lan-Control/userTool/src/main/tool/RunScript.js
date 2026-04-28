import { exec } from 'child_process'
import { printLog } from './log'
import { join } from 'path'
import { sendMessage } from './GetWs'
import { existsSync, writeFileSync } from 'fs'
import { GetDefaultConfig } from './getSystemInfo'
import { createVncIniFile } from './ReadJsonFile'
import { checkService } from './syncFiles'
import { getCurrentTimeString } from './getSystemInfo'
import ConfigManager from './Config'
import { requireJsonFile } from './ReadJsonFile'

let FilePath = null
let configPath = null
let win = null
let UltraVNCPath = null
let UltraVNCConfig = null

const startUltraVNC = (filepath) => {
  return new Promise((resolve) => {
    exec(filepath, (error, stdout, stderr) => {
      if (error) {
        resolve(false)
        return
      }
      resolve(true)
    })
  })
}

const initGetPath = () => {
  FilePath = ConfigManager.get('FilePath')
  configPath = ConfigManager.get('configPath')
  win = ConfigManager.get('win')
  UltraVNCPath = ConfigManager.get('UltraVNCPath')
  UltraVNCConfig = ConfigManager.get('UltraVNCConfig')
}

const handleCommand = async (command, configKey = '') => {
  printLog('[GetWs.js]执行指令:', command)

  const sendResult = (sta, title, text) => sendMessage({ type: 'script', sta, title, text })

  const runCommand = async (cmd) => {
    const res = await startUltraVNC(cmd)
    sendResult(res, res ? '执行成功' : '执行失败', res ? '脚本执行成功' : '执行失败...请稍后重试。')
    return res
  }

  switch (command) {
    case 'showdown':
      printLog('执行关机命令')
      await runCommand('shutdown /s /t 0')
      break

    case 'restart':
      printLog('执行重启命令')
      await runCommand('shutdown /r /t 0')
      break

    case 'furmark': {
      printLog('启动furmark')
      const furmarkPath = join(FilePath, './lib/FurMark_win64/furmark.exe')
      if (existsSync(furmarkPath)) {
        await startUltraVNC(
          `${furmarkPath} --demo furmark-gl --width 1920 --height 1080 --fullscreen`
        )
        sendResult(true, '执行成功', '执行furmark成功')
      } else {
        win.webContents.send('tips', `${getCurrentTimeString()} furmark文件丢失，请检查路径。`)
        sendResult(false, '执行失败', 'furmark文件丢失，请检查路径。')
      }
      break
    }

    case 'changeName': {
      try {
        let nickname = configKey || `keduoli_${Date.now()}`
        ConfigManager.set('Config.nikename', nickname)
        writeFileSync(configPath, JSON.stringify(ConfigManager.get('Config'), null, 2))
        sendMessage({ type: 'nickname', nickname })
        win.webContents.send('Systeminfo', { nikename: nickname, state: true })
        GetDefaultConfig()
      } catch (error) {
        win.webContents.send('tips', `${getCurrentTimeString()}修改昵称失败: ${error.message}`)
        sendResult(false, '修改昵称失败', `修改昵称失败: ${error.message}`)
      }
      break
    }

    case 'getInfo':
      let getNewScriptDate = requireJsonFile(ConfigManager.get('configPath'))
      sendMessage({
        type: 'info',
        configInfo: ConfigManager.get('Config').script || getNewScriptDate.script
      })
      break

    case 'verifyPassword': {
      if (!ConfigManager.get('Config')?.password) return
      // 校验密码不相等则修改同步密码
      let password = configKey || '66C46F42995462527C'
      if (ConfigManager.get('Config').password !== password) {
        ConfigManager.get('Config').password = password
        writeFileSync(configPath, JSON.stringify(ConfigManager.get('Config'), null, 2))
        try {
          await startUltraVNC(`taskkill /IM winvnc.exe /F`)
        } catch (error) {
          win.webContents.send('tips', `${getCurrentTimeString()}密码操作失败: ${error.message}`)
          sendResult(false, '密码操作失败', `密码操作失败: ${error.message}`)
        }
        await createVncIniFile(UltraVNCConfig, ConfigManager.get('Config').password)
        const command = `"${UltraVNCPath}" -run -config "${UltraVNCConfig}"`
        startUltraVNC(command)
      }
      break
    }

    case 'syncScript':
      ConfigManager.set('syncSta', false)
      let res = await checkService()
      win.webContents.send('tips', `${getCurrentTimeString()} 开始同步脚本.....`)
      sendMessage({ type: 'syncScript', data: res })
      break

    default: {
      const script = ConfigManager.get('Config').script[command]
      if (script && existsSync(script.path)) {
        const res = await runCommand(`start ${script.path} ${script?.code ? script.code : ''}`)
        if (!res) {
          delete ConfigManager.get('Config').script[command]
          writeFileSync(configPath, JSON.stringify(ConfigManager.get('Config'), null, 2))
          sendMessage({ type: 'info', configInfo: ConfigManager.get('Config').script })
        }
      } else {
        win.webContents.send(
          'tips',
          `${getCurrentTimeString()}执行失败:路径为 ${script?.path || '未知'}，请检查脚本路径.`
        )
        sendResult(false, '执行失败', `执行失败路径为 ${script?.path || '未知'}，请检查脚本路径。`)
        delete ConfigManager.get('Config').script[command]
        writeFileSync(configPath, JSON.stringify(ConfigManager.get('Config'), null, 2))
        sendMessage({ type: 'info', configInfo: ConfigManager.get('Config').script })
      }
    }
  }
}

function runCommand(command) {
  return new Promise((resolve) => {
    exec(command, { encoding: 'utf8' }, (error, stdout, stderr) => {
      if (error) {
        printLog(error, 1010)
        win.webContents.send('tips', `${getCurrentTimeString()}执行失败: ${error.message}`)
        resolve({ sta: false, title: '执行失败', text: error.message })
      } else {
        resolve({ sta: true, title: '执行成功', text: '文件选择完毕' })
      }
    })
  })
}

export { initGetPath, startUltraVNC, handleCommand, runCommand }
