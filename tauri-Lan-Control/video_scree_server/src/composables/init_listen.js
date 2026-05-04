import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ElNotification } from "element-plus";
import { ref } from "vue";
import { useAppConfig } from "@/store/config";

const add_remove_tips = (state, title, mess) =>
  ElNotification({
    type: state,
    title: title,
    message: mess,
    duration: 3000,
  });

export const init_listen = async () => {
  // 上一个通知
  const up_tips_ip = ref(null);
  const AppConfig = useAppConfig();
  let isFirstDevice = true; // 标记是否是第一个设备

  // 监听ws的移除 / 删除
  await listen("devices_event", (e) => {
    let { devices, event } = e.payload;

    switch (event) {
      case "devices_all": {
        // 全部设备
        if (!devices) return;
        if (!AppConfig.device_list_all) {
          AppConfig.device_list_all = devices;
          add_remove_tips("success", "获取设备", "已获取最新的所有设备。");
          isFirstDevice = false;
          return;
        }

        // 更新所有设备状态（包括已存在的设备）
        let hasChanges = false;
        for (const item in devices) {
          const existingDevice = AppConfig.device_list_all[item];
          const newDevice = devices[item];
          
          // 如果设备不存在或状态有变化，则更新
          if (!existingDevice || 
              existingDevice.device_name !== newDevice.device_name ||
              existingDevice.device_ip !== newDevice.device_ip ||
              existingDevice.device_sync !== newDevice.device_sync) {
            AppConfig.device_list_all[item] = newDevice;
            hasChanges = true;
          }
        }

        // 移除不存在的设备
        for (const item in AppConfig.device_list_all) {
          if (!(item in devices)) {
            delete AppConfig.device_list_all[item];
            hasChanges = true;
          }
        }

        // 如果有变化，可以记录日志
        if (hasChanges) {
          console.log("设备列表已更新");
        }
        
        break;
      }
      case "device_add": {
        // 如果是第一个设备或设备列表为空，初始化
        if (!AppConfig.device_list_all) {
          AppConfig.device_list_all = {};
        }

        // 检查是否是第一个设备（列表为空）
        const isEmpty = Object.keys(AppConfig.device_list_all).length === 0;

        AppConfig.device_list_all[devices[0]] = devices[1];

        // 如果是第一个设备，延时1秒后尝试获取全部设备
        if (isFirstDevice || isEmpty) {
          isFirstDevice = false;
          console.log("第一个设备连接，准备触发设备列表更新");

          // 延时1秒后触发设备列表更新
          setTimeout(async () => {
            try {
              console.log("触发设备列表更新...");
              // 调用后端获取所有设备
              const allDevices = await invoke("sync_devices");
              console.log("获取到的设备列表:", allDevices);

              // 更新设备列表
              AppConfig.device_list_all = allDevices;

              // 触发通知
              add_remove_tips(
                "success",
                "设备列表更新",
                "已获取所有连接的设备",
              );
            } catch (error) {
              console.error("获取设备列表失败:", error);
            }
          }, 1000);
        }

        if (up_tips_ip.value !== devices[0]) {
          add_remove_tips("success", "新增设备", `连接到设备: ${devices[0]}`);
          up_tips_ip.value = devices[0];
        }
        break;
      }
      case "device_remove": {
        if (AppConfig.device_list_all && AppConfig.device_list_all[devices]) {
          delete AppConfig.device_list_all[devices];
          add_remove_tips(
            "error",
            "设备断开",
            `设备: ${devices}, 断开了链接。`,
          );
        }
        break;
      }

      default:
        console.log("发生错误/无法识别", e.payload);
    }
  });

  await listen("device_config", async (e) => {
    console.log("收到device_config事件:", e.payload);
    
    // 检查是否是特殊消息（如request_file_sync）
    if (typeof e.payload === 'string') {
      const trimmed = e.payload.trim();
      if (trimmed === 'request_file_sync') {
        console.log("收到文件同步请求");
        // 触发文件同步操作
        try {
          await invoke('trigger_file_sync');
          console.log('已触发文件同步');
        } catch (error) {
          console.error('触发文件同步失败:', error);
        }
        return;
      }
    }
    
    // 处理其他类型的消息
    if (typeof e.payload !== 'string') {
      console.log("device_config事件不是字符串类型:", e.payload);
      return;
    }
    
    const message = e.payload;
    
    // 根据不同消息类型进行处理
    if (message === 'request_file_sync') {
      // 已经在上面处理过了，这里不需要重复处理
      return;
    }
    
    // 检查消息格式：使用管道符分割
    if (!message.includes('|')) {
      console.log("device_config事件格式不正确(缺少|分隔符):", message);
      return;
    }
    
    const parts = message.split('|');
    const eventType = parts[0];
    
    // 根据事件类型处理不同的消息格式
    switch (eventType) {
      case "device_sync": {
        // 设备同步状态事件
        console.log("设备同步状态消息:", message);
        
        // 消息格式为 "device_sync|true" 或 "device_sync|false"
        if (parts.length >= 2) {
          const syncStatus = parts[1] === "true";
          console.log(`收到设备同步状态: ${syncStatus}`);
          
          // 注意：这里没有设备IP，但我们可以记录日志
          // 实际设备同步状态通过device_sync_ip事件更新
          console.log(`设备同步状态: ${syncStatus} (IP未知，等待device_sync_ip事件)`);
        }
        break;
      }
      case "device_sync_ip": {
        // 设备同步状态事件（包含IP地址）
        console.log("设备同步状态(含IP)消息:", message);
        
        // 消息格式为 "device_sync_ip|{ip}|{status}"
        if (parts.length >= 3) {
          const deviceIp = parts[1];
          const isSynced = parts[2] === "true";
          
          console.log(`设备 ${deviceIp} 同步状态: ${isSynced}`);
          
          // 直接更新设备列表中的同步状态
          if (AppConfig.device_list_all && AppConfig.device_list_all[deviceIp]) {
            AppConfig.device_list_all[deviceIp].device_sync = isSynced;
            console.log(`已更新设备 ${deviceIp} 的同步状态为: ${isSynced}`);
          } else {
            console.warn(`设备 ${deviceIp} 不在当前设备列表中`);
          }
        } else {
          console.warn("device_sync_ip事件格式不正确:", message);
        }
        break;
      }
      case "device_info": {
        // 设备信息事件
        console.log("设备信息消息:", message);
        
        // 消息格式为 "device_info|{json_data}"
        if (parts.length >= 2) {
          const deviceInfoStr = parts.slice(1).join('|'); // 重新组合JSON字符串
          try {
            let json_data = JSON.parse(deviceInfoStr);
            console.log("解析后的设备信息:", json_data);
            
            let info_key = json_data?.device_key;
            if (!info_key) break;
            
            if (!AppConfig.device_list_all) {
              AppConfig.device_list_all = {};
            }
            
            // 更新设备信息，保留原有的device_sync状态
            const existingDevice = AppConfig.device_list_all[info_key];
            if (existingDevice) {
              // 保留原有的device_sync状态
              json_data.device_sync = existingDevice.device_sync;
            } else {
              // 新设备，如果没有device_sync字段，默认设为false
              if (json_data.device_sync === undefined) {
                json_data.device_sync = false;
              }
            }
            
            AppConfig.device_list_all[info_key] = json_data;
            console.log(`已更新设备 ${info_key} 的信息`);
            
            // 不需要调用后端API，因为设备信息已经通过WS消息从服务器接收
            // 设备信息更新逻辑已经在服务器端的commadn_ws.rs中处理
          } catch (error) {
            console.error("解析设备信息失败:", error, "message:", message);
          }
        }
        break;
      }
      case "script": {
        try {
          // 消息格式为 "script|{json_data}"
          const scriptDataStr = parts.slice(1).join('|'); // 重新组合JSON字符串
          console.log("收到脚本事件:", scriptDataStr);
          let json_data = JSON.parse(scriptDataStr);
          console.log("解析后的json_data:", json_data);
          
          let keys = Object.keys(json_data);
          console.log("脚本键:", keys);
          for (let key of keys) {
            AppConfig.run_script_list[key] = json_data[key];
            const scriptId = json_data[key].id;
            const scriptData = {
              name: json_data[key].name,
              path: json_data[key].path,
              key: scriptId,
              fullData: json_data[key], // 保存完整数据
            };
            
            // 检查脚本是否已经在常驻命令中
            if (AppConfig.option_script[0] && AppConfig.option_script[0][scriptId]) {
              // 更新常驻命令中的脚本信息
              AppConfig.option_script[0][scriptId] = scriptData;
              console.log(`脚本 ${scriptId} 已在常驻命令中，已更新`);
            } else {
              // 检查是否已经在可选命令中，避免重复添加
              if (!AppConfig.option_script[1] || !AppConfig.option_script[1][scriptId]) {
                // 添加到可选命令
                if (!AppConfig.option_script[1]) {
                  AppConfig.option_script[1] = {};
                }
                AppConfig.option_script[1][scriptId] = scriptData;
                console.log(`脚本 ${scriptId} 已添加到可选命令`);
              } else {
                // 更新可选命令中的脚本信息
                AppConfig.option_script[1][scriptId] = scriptData;
                console.log(`脚本 ${scriptId} 已在可选命令中，已更新`);
              }
            }
          }
          console.log("更新后的run_script_list:", AppConfig.run_script_list);
          console.log("更新后的option_script:", AppConfig.option_script);
        } catch (error) {
          console.error("解析脚本事件失败:", error, "message:", message);
        }
        break;
      }
      case "script_executed": {
        // 脚本执行结果事件
        console.log("脚本执行结果消息:", message);
        
        // 消息格式为 "script_executed|{script_id}|{result}"
        if (parts.length >= 3) {
          const scriptId = parts[1];
          const result = parts[2];
          add_remove_tips(
            result === "success" ? "success" : "error",
            "脚本执行结果",
            `脚本 ${scriptId} 执行${result === "success" ? "成功" : "失败"}`,
          );
        }
        break;
      }
      case "request_file_sync": {
        // 文件同步请求事件
        console.log("文件同步请求消息:", message);
        
        // 消息格式为 "request_file_sync|{missing_files}"
        if (parts.length >= 2) {
          const missing_files_str = parts[1];
          console.log("文件同步请求，缺失文件:", missing_files_str);
          
          // 这里可以触发UI上的文件同步进度显示
          // 或者记录日志，但不需要具体处理，因为后端已经处理了
          console.log("后端已收到文件同步请求，正在同步文件...");
        }
        break;
      }
      case "vnc_download": {
        // VNC下载状态事件
        console.log("VNC下载状态消息:", message);
        
        // 消息格式为 "vnc_download|{ip}|{status}"
        if (parts.length >= 3) {
          const deviceIp = parts[1];
          const status = parts.slice(2).join('|'); // 合并剩余部分（可能包含冒号）
          
          console.log(`设备 ${deviceIp} VNC下载状态: ${status}`);
          
          // 显示通知
          let title = "VNC下载状态";
          let msg = "";
          let type = "info";
          
          if (status === "start") {
            msg = `设备 ${deviceIp} 正在下载UltraVNC...`;
            type = "info";
          } else if (status === "success") {
            msg = `设备 ${deviceIp} UltraVNC下载并解压成功！`;
            type = "success";
          } else if (status.startsWith("failed:")) {
            const errorMsg = status.replace("failed:", "");
            msg = `设备 ${deviceIp} UltraVNC下载失败: ${errorMsg}`;
            type = "error";
          } else {
            msg = `设备 ${deviceIp} VNC下载状态: ${status}`;
          }
          
          add_remove_tips(type, title, msg);
          
          // 如果是下载开始或失败，应该关闭打开的vnc-window.html窗口
          if (status === "start" || status.startsWith("failed:")) {
            console.log(`VNC下载状态 ${status}，应该关闭vnc-window.html窗口`);
            // 这里可以触发关闭vnc-window.html窗口的逻辑
            // 由于无法直接访问DOM，可以通过事件或状态管理来通知UI组件
          }
        } else {
          console.warn("vnc_download事件格式不正确:", message);
        }
        break;
      }
      case "vnc_started": {
        // VNC启动状态事件
        console.log("VNC启动状态消息:", message);
        
        // 消息格式为 "vnc_started|{ip}|{status}"
        if (parts.length >= 3) {
          const deviceIp = parts[1];
          const status = parts[2];
          
          console.log(`设备 ${deviceIp} VNC启动状态: ${status}`);
          
          // 显示通知
          let title = "VNC启动状态";
          let msg = "";
          let type = "info";
          
          if (status === "success") {
            msg = `设备 ${deviceIp} UltraVNC已成功启动！`;
            type = "success";
          } else {
            msg = `设备 ${deviceIp} VNC启动状态: ${status}`;
          }
          
          add_remove_tips(type, title, msg);
        } else {
          console.warn("vnc_started事件格式不正确:", message);
        }
        break;
      }
      default: {
        console.log("未识别的事件类型:", eventType, "完整消息:", message);
      }
    }
  });
};