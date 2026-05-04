import { defineStore } from "pinia";
import { ref } from "vue";

export const useAppConfig = defineStore("AppConfig", () => {
  // 窗口状态 true - 最大化 / false - 窗口
  const isMaxWindow = ref(false);
  // 所有设备数据
  const device_list_all = ref(null);
  // 可执行脚本列表
  const run_script_list = ref({});
  // 常驻命令
  const option_script = ref([
    {
      shutdown: { name: "全部关机", path: "shutdown", key: "shutdown" },
      reboot: { name: "全部重启", path: "reboot", key: "reboot" },
      sync_files: { name: "同步文件", path: "sync_files", key: "sync_files" },
    },
    {

    }
  ]);
  // 软件配置
  const config = ref({
    autoStart: false,
    wsPort: 9000,
    broadcastPort: 13140,
    preview: true,
    pausePreview: false,
    reconnectWs: true,
    reconnectTimes: 5,
    reconnectInterval: 2,
  });

  return {
    isMaxWindow,
    device_list_all,
    run_script_list,
    config,
    option_script,
  };
});
