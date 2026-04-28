import { ref } from 'vue'
import { ElNotification } from 'element-plus'

export const modelShow = ref(false) // 弹窗状态
export const showModelPage = ref(true) // 显示设置页面/弹窗
export const ActiveIp = ref('') // 当前活动IP
export const ActiveSyncSta = ref(false) // 当前同步状态
export const isFull = ref(false)
export const loading = ref(false)
// 当前设备列表
export const device_list = ref([
  {
    ip: '192.168.110.2',
    name: 'test',
    syncSta: true
  }
])
// 日志打印
const logMap = {
  error: console.error,
  warning: console.warn,
  info: console.info,
  success: console.info,
  debug: console.debug
}
function log(level = 'info', message = '') {
  console.log(level)
  ;(logMap[level] || console.log)(`[${level.toUpperCase()}]`, message)
}

// 弹窗通知
export const ElNotificationFn = (sta, title, message, duration = 3000) => {
  log(sta || 'info', message)
  ElNotification({
    type: sta || 'info',
    title: title,
    message: message,
    duration
  })
}

// 执行远程查看
export const remoteView = async (ip) => {
  const res = await window.api.remoteView(ip)
  if (res[0]) {
    ElNotificationFn('success', '连接成功', '你可以开始远程查看/控制了')
  } else {
    ElNotificationFn('error', '连接失败', res[1] || '连接失败，请检查IP地址')
  }
}

// 初始化监听器
export const initListeners = () => {
  window.api.scriptResultFn('scriptResult', (data) => {
    // 根据data.sta修改弹窗状态
    ElNotificationFn(
      data?.sta ? 'success' : 'error',
      data?.title || '操作失败',
      data?.text || '操作失败，设备未响应',
      data?.duration || 3000
    )
  })
}
// 执行关机/重启/命令等
export const SendRunScript = async (script) => {
  if (script[1] === 'getInfo') loading.value = true
  let res = await window.api.command(script)

  setTimeout(() => {
    loading.value = false
  }, 1000)
}

export const checkJsonData = (data) => {
  let jsonData

  if (typeof data === 'object' && data !== null) {
    jsonData = data
  } else if (typeof data === 'string') {
    try {
      const parsed = JSON.parse(data)
      if (typeof parsed === 'object' && parsed !== null) {
        jsonData = parsed
      } else {
        console.warn('解析后的数据不是对象:', parsed)
        jsonData = {}
      }
    } catch (err) {
      console.error('JSON 解析失败:', err)
      jsonData = {}
    }
  } else {
    console.warn('无效的数据类型:', typeof data)
    jsonData = {}
  }

  return jsonData
}

// 发送同步脚本信息
export const syncScript = async () => {
  loading.value = true
  let res = await window.api.command(['', 'syncScript'])
  ElNotificationFn('success', '发送成功', '已广播同步脚本消息，等待客户端回复...')
  setTimeout(() => {
    loading.value = false
  }, 1500)
}

// 防抖
export const debounce = (fn, delay = 300) => {
  let timer = null
  return function (...args) {
    clearTimeout(timer)
    timer = setTimeout(() => {
      fn.apply(this, args)
    }, delay)
  }
}
