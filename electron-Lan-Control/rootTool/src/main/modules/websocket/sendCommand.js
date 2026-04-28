import { join, basename } from 'path'
import { writeFileSync, existsSync } from 'fs'
import { validatePassword } from '../vnc/validatePassword'
import { getClient, getAllClients } from '../websocket/clients'
import { openUrl, selectFileOrFolder } from '../script'
import { tarFileZipToPath, copyFile } from '../config'
import { getFilePath, getScriptList } from '../config/getConfig'

// 发送执行方法和返回在线设备列表
const sendCommand = (ws, command, nikename = '') => {
  let obj = nikename
    ? { type: 'command', payload: command, nikename }
    : { type: 'command', payload: command }
  ws.send(JSON.stringify(obj))
}

const sendCommandFn = async (command) => {
  const ws = getClient(command[0])?.ws
  if (command[1] === 'verifyPassword') {
    try {
      validatePassword(command[0])

      return {
        sta: true,
        text: '校验密码/修改密码成功'
      }
    } catch (error) {
      return {
        sta: false,
        text: error.message
      }
    }
  } else if (command[1] === 'changeName') {
    if (ws) {
      let nikename = command[2] || ''
      sendCommand(ws, command[1], nikename)

      return {
        sta: true,
        text: '已发送修改昵称'
      }
    } else {
      return {
        sta: false,
        text: '未找到该设备'
      }
    }
  } else if (command[1] === 'openLink') {
    openUrl(command[0])
  } else if (command[1] === 'select') {
    let res = await selectFileOrFolder(command[0])

    return res
  } else if (command[1] === 'scriptFile') {
    let res = null
    let config = null

    if (command[0]) config = JSON.parse(command[0])
    const fileName = basename(config.filePath)
    const toFile = join(getFilePath(), './data')
    const zip7 = join(getFilePath(), './lib/7zip/7za.exe')
    const scriptConfig = join(toFile, './config.json')
    const targetName = fileName + '.zip'

    if (!existsSync(scriptConfig))
      writeFileSync(scriptConfig, JSON.stringify({ files: [] }, null, 2))

    if (config.fielOrDoc === 1) res = await copyFile(config.filePath, join(toFile, fileName))
    if (config.fielOrDoc === 2)
      res = await tarFileZipToPath(config.filePath, join(toFile, targetName), zip7)

    if (res.sta) {
      let scriptList = getScriptList() || { files: [] }
      res.sta
        ? scriptList.files.push({
            title: config.filename,
            filename: config.fielOrDoc === 2 ? targetName : fileName,
            url: `/files/${config.fielOrDoc === 2 ? targetName : fileName}`,
            code: config.key,
            fielOrDoc: config.fielOrDoc
          })
        : ''
      writeFileSync(join(toFile, './config.json'), JSON.stringify(scriptList, null, 2))
    }
    return res
  } else if (command[1] === 'syncScript') {
    const wsAll = getAllClients()
    for (let [key, value] of wsAll) {
      sendCommand(value.ws, command[1])
    }
    return 1
  } else {
    if (ws) {
      let getInfo = command[2] || ''
      sendCommand(ws, command[1], getInfo)
    }
    return 1
  }
}
export { sendCommand, sendCommandFn }
