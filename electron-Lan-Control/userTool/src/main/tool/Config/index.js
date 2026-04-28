import { requireJsonFile } from '../ReadJsonFile'
import { existsSync, writeFileSync } from 'fs'
import { join } from 'path'

const defaultConfig = {
  nikename: 'keduoli',
  AdPart: 41234,
  wspart: 9191,
  defAdPart: 41234,
  defwspart: 9191,
  password: '66C46F42995462527C',
  autorun: false,
  httpPort: 10240,
  script: {}
}
let isChange = false

const ConfigManager = {
  state: {
    win: null,
    Config: null,
    configPath: null,
    UltraVNCPath: null,
    UltraVNCConfig: null,
    FilePath: null,
    syncSta: false
  },

  init(_FilePath, _win) {
    this.state.FilePath = _FilePath
    this.state.win = _win
    this.state.configPath = join(_FilePath, 'config.json')
    this.state.Config = requireJsonFile(this.state.configPath) || {}
    // 在 init 中调用检查方法
    this.checkConfigFile()
    this.state.UltraVNCPath = join(_FilePath, './lib/ultravnc_x64/winvnc.exe')
    this.state.UltraVNCConfig = join(_FilePath, './lib/ultravnc_x64/runconfig.ini')
  },

  checkConfigFile() {
    isChange = false
    let config
    if (!existsSync(this.state.configPath)) {
      config = { ...defaultConfig }
    } else {
      try {
        config = requireJsonFile(this.state.configPath) || {}
      } catch (err) {
        config = { ...defaultConfig }
      }
      for (const key in defaultConfig) {
        if (key in config) continue
        config[key] = defaultConfig[key]
        isChange = true
      }
    }
    if (isChange) writeFileSync(this.state.configPath, JSON.stringify(config, null, 2))
    this.set('Config', config)
  },

  get(key) {
    if (!(key in this.state)) throw new Error(`未知配置项: ${key}`)
    return this.state[key]
  },

  set(path, value) {
    if (!value) return
    const keys = path.split('.')
    let obj = this.state
    for (let i = 0; i < keys.length - 1; i++) {
      if (!(keys[i] in obj)) obj[keys[i]] = {}
      obj = obj[keys[i]]
    }
    obj[keys[keys.length - 1]] = value
  }
}

export default ConfigManager
