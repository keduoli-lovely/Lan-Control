import { existsSync, writeFileSync } from 'fs'
import { sendMessage } from './GetWs'
import { applyAutorunSetting, AddCloseTask } from './initApp'
import ConfigManager from './Config'
import { join } from 'path'

let FilePath = null
let config = null
let configPath = null
let autoRunFn = null
// 初始化
const initUpdateConfig = () => {
  FilePath = ConfigManager.get('FilePath')
  config = ConfigManager.get('Config')
  configPath = ConfigManager.get('configPath')
}

function saveConfig(successMsg, errorMsg) {
  const getNewConfig = ConfigManager.get('Config')

  if (!config && configPath) return
  try {
    writeFileSync(configPath || getNewConfigPath, JSON.stringify(config || getNewConfig, null, 2))
    return { text: successMsg, state: true }
  } catch (error) {
    return { text: errorMsg + error.message, state: false }
  }
}

function handleData(data, list) {
  switch (data[0]) {
    case 'Add': {
      const batFile = join(FilePath, `./script/${list.file}`)
      if (!existsSync(batFile)) {
        return { text: '未找到脚本文件', state: false }
      }

      if (list.name && list.file) {
        ConfigManager.set('Config.script', config.script || {})
        if (config.script[list.name]) {
          return { text: '脚本名称已存在，请更换名称', state: false }
        }

        ConfigManager.set(`Config.script.${list.name}`, {
          name: list.name,
          path: batFile
        })

        sendMessage({ type: 'info', configInfo: config.script })
        return saveConfig('脚本添加成功', '脚本添加失败')
      }
      break
    }

    case 'update': {
      if (list.AdPart !== '') ConfigManager.set('Config.AdPart', Number(list.AdPart))
      if (list.wspart !== '') ConfigManager.set('Config.wspart', Number(list.wspart))

      return saveConfig('端口修改成功', '端口修改失败')
    }

    case 'setname': {
      ConfigManager.set('Config.nikename', list.name)
      sendMessage({ type: 'nickname', nickname: config.nikename })
      return saveConfig('名称修改成功', '名称修改失败')
    }

    case 'Reset': {
      ConfigManager.set('Config.AdPart', config.defAdPart)
      ConfigManager.set('Config.wspart', config.defwspart)
      return saveConfig('端口重置成功', '端口重置失败')
    }

    case 'autorun': {
      config.autorun = list?.RunValue || false
      ConfigManager.set('Config.autorun', list?.RunValue || false)
      setRunConfig()
      return saveConfig('开启自启动', '关闭自启动')
    }

    default:
      return { text: '未知操作', state: false }
  }
}

// 等待修改自启动事件
const setRunConfig = () => {
  if (autoRunFn) clearTimeout(autoRunFn)
  autoRunFn = setTimeout(() => {
    applyAutorunSetting(ConfigManager.get('Config').autorun)
  }, 2000)
}

AddCloseTask(() => {
  autoRunFn && clearTimeout(autoRunFn)
})

export { initUpdateConfig, handleData }
