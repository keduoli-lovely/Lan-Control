<script setup>
import { ref, onMounted } from "vue";
import { Store } from '@tauri-apps/plugin-store';
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ElNotification } from 'element-plus'
import { TrayIcon } from '@tauri-apps/api/tray';
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { defaultWindowIcon } from '@tauri-apps/api/app';
import { Menu } from '@tauri-apps/api/menu';

// 存放store / text
let store = null
let win = null
const script1 = ref({})
const auto_state = ref(false)
const close_state = ref(false)
const dev_name = ref("keduoli")
const timer = ref(null)

const change_auto_state = async (value) => {
  if (timer.value) {
    clearTimeout(timer.value)
  }
  timer.value = setTimeout(async () => {
    await invoke("set_auto_run", { state: value })
    await store.set("auto_state", value)
    await store.save()
  }, 1500)
}

onMounted(async () => {
  // 获取窗口
  win = await WebviewWindow.getCurrent();
  store = await Store.load('.settings.json');
  const store_name = await store.get("dev_name")
  const store_close_state = await store.get("close_state")
  const store_auto_state = await store.get("auto_state")
  dev_name.value = store_name || "keduoli"
  close_state.value = store_close_state || false
  auto_state.value = store_auto_state || false
  await listen('command', async (e) => {
    const conten_list = e?.payload ? e.payload.split('|') : [e.payload]
    switch (conten_list[0]) {
      // 开启远程
      case 'see': {
        // await invoke("run_cap");
        break;
      }
      case 'stop':
        // 停止远程
        await invoke("stop_cap")
        break;
      case 'key': {
        // 设置唯一id， 并将设备详情发送给服务端
        // 设置服务端给定的key， 默认ip 
        await invoke("send_device_info", { deviceKey: conten_list[1], deviceName: dev_name.value })
        break;
      }
      case 'script': {
        // 执行服务端发送的命令
        await invoke("run_script", { filePath: conten_list[1], code: conten_list[2] })
        break;
      }
      case 'change_name': {
        // 修改客户端显示名称
        let tmp_name = conten_list[1] || "keduoli"
        dev_name.value = tmp_name
        await store.set("dev_name", tmp_name)
        await store.save()
        break;
      }
      case 'get_script': {
        // 获取客户端可执行脚本
        await invoke("send_script", {
          script: script1.value
        })
        break;
      }
      case 'close_ws': {
        // 关闭所有WebSocket连接
        await invoke("close_all_ws")
        break;
      }
      case 'restart_broadcast': {
        // 重启广播服务，重新发现服务器
        console.log("收到重启广播指令，重新初始化运行时...")
        await initialize_ws_run(true)  // 传入true表示是重启过程
        break;
      }
      case 'auto_run_state': {
        // 设置开启自启动状态
        console.log(conten_list)
        try {
          auto_state.value = JSON.parse(conten_list[1])
        } catch (error) {
          auto_state.value = false
        }

        await change_auto_state(auto_state.value)
        break;
      }
      default: {
        console.log("指令错误", e.payload)
        return;
      }
    }
  })

  win?.listen("tauri://close-requested", async () => {
    if (close_state.value) {
      // 最小化到托盘
      win.hide()
    } else {
      // 退出应用
      win.destroy()
    }
    store.set("close_state", close_state.value)
    store.save()
  })
  await initialize_ws_run()
  await TrayIcon.new({
    // 设置托盘和双击显示
    icon: await defaultWindowIcon(),
    tooltip: "双击显示界面",
    menu: await Menu.new({
      // 显示界面，退出应用
      items: [
        {
          id: "show",
          text: "显示界面",
          action: () => {
            win.show()
          }
        },
        {
          id: "exit",
          text: "退出应用",
          action: () => {
            win.destroy()
          }
        }
      ]
    }),
    action: (event) => {
      if (event.type === 'DoubleClick') {
        win.show()
      }
    }
  })
})

const initialize_ws_run = async (isRestart = false) => {
  const maxRetries = isRestart ? 10 : 1;
  const retryDelay = 2000; // 2秒

  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    let runtime_state = await invoke("initialize_runtime")

    if (runtime_state) {
      if (isRestart && attempt > 1) {
        console.log(`重启广播成功，第${attempt}次尝试`)
      }
      return true;
    }

    // 如果不是最后一次尝试，等待后继续
    if (attempt < maxRetries) {
      console.log(`初始化运行时失败，等待${retryDelay / 1000}秒后重试 (${attempt}/${maxRetries})`)
      await new Promise(resolve => setTimeout(resolve, retryDelay))
    } else {
      // 最后一次尝试失败
      if (!isRestart) {
        ElNotification({
          type: 'Error',
          title: 'tips',
          message: "ip或端口获取失败, 请重启应用或检查服务端日志",
          duration: 3000
        })
      } else {
        console.log(`重启广播失败，已重试${maxRetries}次`)
      }
      return false;
    }
  }
}
</script>

<template>
  <main class="container">
    <div class="setting_page">
      <div class="dev_name">
        <div class="name_text">设备名称: </div>
        <div class="name_vlaue">
          <el-input v-model="dev_name" readonly style="width: 120px" placeholder="设置名称." />
        </div>
      </div>
      <div class="auto_run">
        <div class="text">开机自启: </div>
        <el-switch v-model="auto_state" @change="change_auto_state" />
      </div>

      <div class="close_btn">
        <div class="text_close">关闭按钮:</div>
        <div class="text_sel">
          <div class="text1" :class="!close_state ? 'text_sel_atv' : ''" @click="close_state = false">退出应用</div>
          <div class="mark"></div>
          <div class="text2" :class="close_state ? 'text_sel_atv' : ''" @click="close_state = true">最小化应用</div>
        </div>
      </div>
    </div>
  </main>
</template>

<style scoped>
.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #249b73);
}
</style>
<style scoped>
html,
body {
  width: 100%;
  height: 100%;
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

.container {
  width: 100%;
  height: 100%;
}

.dev_name {
  display: flex;
  align-items: center;
}

.name_text {
  margin-right: 5px;
}

.auto_run {
  display: flex;
  align-items: center;
  margin-top: 4px;
}

.text,
.text_close {
  margin-right: 5px;
}

.text_sel,
.close_btn {
  display: flex;
}

.mark {
  width: 14px;
}

.text_sel {
  padding: 2px;
  border: 1px solid #ccc;
  border-radius: 4px;
}

.text_sel_atv {
  background-color: skyblue;
  border-radius: 4px;
}

.text1,
.text2 {
  cursor: pointer;
  padding: 1px 4px;
}
</style>