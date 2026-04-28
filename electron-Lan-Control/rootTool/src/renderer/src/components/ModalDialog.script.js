import { ref, watch, onMounted, nextTick } from 'vue'
import {
  modelShow,
  SendRunScript,
  device_list,
  debounce,
  loading,
  ElNotificationFn
} from '../utils/popupStore'

export const form = ref({
  AdPart: null,
  wspart: null,
  LinkPwd: '',
  autorun: false,
  BROADCAST_INTERVAL: 5000,
  ADRunTime: 3000,
  HttpSta: false,
  HttpPort: 10240
})
export const AppConfig = ref({})
export const saveConfig = ref(false)
export const changeList = ref({})
export const dialogRef = ref(null)
export const scriptListData = ref({})
export const sendBtn = ref(false)
export const SetWindowSize = ref({
  max: {
    width: '80vw',
    height: '57vh'
  },
  min: {
    width: '30vw',
    height: '27vh'
  }
})
export const fileObj = ref({
  filename: '',
  filePath: '',
  key: '',
  fielOrDoc: 0
})
const onceShow = ref(true)

export const checkDeviceStatus = (ip, name) => {
  const device = device_list.value.find((item) => item.ip === ip)
  if (device) {
    SendRunScript([ip, name])
  } else {
    dialogRef.value?.classList.remove('open')
    setTimeout(() => {
      dialogRef.value?.close()
      ElNotificationFn('warning', '设备异常', '设备离线/设备异常', 4000)
    }, 300)
  }
}

export const saveSettings = async () => {
  const HttpSta = form.value.HttpSta
  loading.value = true
  form.value[['autorun']] ? '' : delete changeList.value['autorun']
  if (changeList.value) {
    let res = await window.api.saveNewConfig(JSON.stringify(changeList.value))
    form.value = { ...res.data, HttpSta }
    AppConfig.value = { ...res.data, HttpSta }
    changeList.value = {}
    saveConfig.value = false
    setTimeout(() => {
      loading.value = false
      ElNotificationFn(
        res.sta ? 'success' : 'error',
        res.sta ? '操作成功' : '操作失败',
        res.text || '操作失败，设备未响应'
      )
    }, 800)
  }
}

function getDiff(formData, configData) {
  const result = {}
  for (const key in formData) {
    if (key in configData && !Object.is(formData[key], configData[key])) {
      result[key] = { old: configData[key], new: formData[key] }
    }
  }
  return result
}

export const setupWatchers = () => {
  onMounted(() => {
    watch(modelShow, async (val) => {
      if (val) {
        if (onceShow.value) {
          AppConfig.value = await window.api.GetConfig()
          onceShow.value = false
          Object.assign(form.value, AppConfig.value)
        }
        await nextTick()
        dialogRef.value?.showModal()
        dialogRef.value?.classList.add('open')
      } else {
        dialogRef.value?.classList.remove('open')
        setTimeout(() => {
          dialogRef.value?.close()
        }, 300)
      }
    })

    watch(
      form,
      debounce((newVal) => {
        const changes = getDiff(newVal, AppConfig.value)
        const changedKeys = Object.keys(changes)
        if (!changedKeys.length) {
          saveConfig.value = false
          return
        }

        let valid = true

        for (const key of changedKeys) {
          const { new: newValue, old: oldValue } = changes[key]
          if (Object.is(newValue, oldValue)) {
            valid = false
            break
          }
          if (newValue === '' || newValue === null || newValue === undefined) {
            valid = false
            break
          }

          if ((key === 'AdPart' || key === 'wspart') && (newValue < 2000 || newValue > 65534)) {
            valid = false
            break
          }
        }
        saveConfig.value = valid

        if (valid) {
          changedKeys.forEach((key) => {
            changeList.value[key] = changes[key].new
          })
        }
      }, 300),
      { deep: true }
    )

    watch(
      fileObj,
      (newValue, oldValue) => {
        if (fileObj.value.filePath && fileObj.value.filename) {
          sendBtn.value = true
        } else {
          sendBtn.value = false
        }
      },
      { deep: true }
    )

    window.api.ScriptListFn('ScriptList', (data) => {
      scriptListData.value = data
    })
  })
}

export const rules = {
  AdPart: [
    { required: true, message: '请输入广播端口', trigger: 'change' },
    {
      validator: (rule, value, callback) => {
        const port = Number(value)
        if (!value || port < 2000 || port > 65534) {
          callback(new Error('端口必须在 2000~65534 范围内'))
        } else {
          callback()
        }
      },
      trigger: 'change'
    }
  ],
  wspart: [
    { required: true, message: '请输入 WS 端口', trigger: 'change' },
    {
      validator: (rule, value, callback) => {
        const port = Number(value)
        if (!value || port < 2000 || port > 65534) {
          callback(new Error('端口必须在 2000~65534 范围内'))
        } else {
          callback()
        }
      },
      trigger: 'change'
    }
  ],
  LinkPwd: [
    { required: true, message: '请输入连接密码', trigger: 'change' },
    {
      pattern: /^[A-Za-z\d!@#$%^&*_.]{1,7}$/,
      message: '密码需为1~7位',
      trigger: 'change'
    }
  ]
}

// 选择文件夹
export const selectFile = async (list) => {
  loading.value = true
  let res = await window.api.command(list)

  setTimeout(() => {
    if (res.sta) {
      fileObj.value.filePath = res.path
    }
    ElNotificationFn(
      res.sta ? 'success' : 'error',
      res.sta ? '操作成功' : '操作失败',
      res.text || '操作失败，设备未响应'
    )
    fileObj.value.fielOrDoc = res.sta ? (list[0] ? 1 : 2) : 0
    loading.value = false
  }, 800)
}

// 提交执行脚本
export const sendScript = async () => {
  if (fileObj.value.filePath && fileObj.value.filename) {
    loading.value = true
    let res = await window.api.command([JSON.stringify(fileObj.value), 'scriptFile'])

    ElNotificationFn(
      res.sta ? 'success' : 'error',
      res.sta ? '操作成功' : '操作失败',
      res.text || '操作失败，设备未响应'
    )

    res.sta
      ? (fileObj.value = {
          filename: '',
          filePath: '',
          key: '',
          fielOrDoc: 0
        })
      : ''
    setTimeout(() => {
      loading.value = false
    }, 800)
  }
}
