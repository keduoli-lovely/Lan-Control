import { ref, nextTick } from 'vue'
import { checkJsonData, device_list } from '../utils/popupStore'
import { ElNotification } from 'element-plus'

export const device_pic = ref({})
export const menuShow = ref(false)
export const setEdit = ref({})
export const inputRefs = ref([])
export const moveToOutShowFlag = ref(false)

export const SendToAll = async (script) => {
  if (device_list.value.length > 0) {
    for (let key in device_list.value) {
      let ip = device_list.value[key].ip
      await window.api.command([ip, script])
    }
  } else {
    ElNotification({
      type: 'warning',
      title: '提示',
      message: '没有可用设备',
      duration: 3000
    })
  }

  menuShow.value = !menuShow.value
  setTimeout(() => {
    moveToOutShowFlag.value = false
  }, 500);
}

export const handleClick = (event) => {
  const targetName = event.target.dataset?.keduoli
  if (!targetName) return
  if (targetName === 'showDom') {
    menuShow.value = !menuShow.value
  }
  event.stopPropagation()
}

export const setItemRef = (index) => {
  return el => {
    inputRefs.value[index] = el
  }
}

export const enableEdit = (item, index) => {
  ElNotification({
    type: 'success',
    title: '编辑名称',
    message: '使用回车/空白处点击即可应用新名称',
    duration: 3000
  })
  setEdit.value[item.ip] = {
    state: true,
    newName: ''
  }
  nextTick(() => {
    inputRefs.value[index]?.focus()
  })
}

export const checkNameChange = async (item) => {
  const newName = setEdit.value[item.ip]?.newName
  if (item.name === newName || !newName) {
    setEdit.value[item.ip].state = false
    ElNotification({
      type: 'warning',
      title: 'tips',
      message: '未修改名称',
      duration: 3000
    })
    return
  }

  item.name = newName
  setEdit.value[item.ip].state = false
  await window.api.command([item.ip, 'changeName', newName])
  ElNotification({
    type: 'success',
    title: 'tips',
    message: '已修改名称',
    duration: 3000
  })
}

export const setupListeners = () => {
  window.api.InfoDataList('message', (data) => {
    const jsonData = checkJsonData(data)

    if (jsonData.type === 'newClient') {
      for (let key in setEdit.value) {
        if (!setEdit.value[key]) {
          delete setEdit.value[key]
        }
      }
      device_list.value = jsonData.data || []
      console.log(device_list.value)
    } else if (jsonData.type === 'DesktopJpg') {
      device_pic.value = jsonData.data || {}
    }
  })
}

const moveToOutTimer = ref(null)
export const moveTOShowMenu = (type) => {
  if(type === 'move') {
    if(moveToOutTimer.value) {
      clearTimeout(moveToOutTimer.value)
    }

    moveToOutShowFlag.value = true
  }else if(type === 'out') {
    if(menuShow.value) return
    moveToOutTimer.value = setTimeout(() => {
      moveToOutShowFlag.value = false
    }, 5000);
  }
}