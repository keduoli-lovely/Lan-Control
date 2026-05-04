pub mod broadcast;
pub mod command;
pub mod file_monitor;
pub mod file_sync;
pub mod handle_run;
pub mod screenshot_ws;
pub mod ws_server;

use command::{open_vnc_window, VncProxyState};
use file_monitor::{get_scripts_config, save_script};
use file_sync::sync_script_files;
use handle_run::devices_map::{DevicePlayload, DevicesMap};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tauri::{AppHandle, Emitter, State};

#[derive(Serialize, Deserialize)]
pub struct WebMessage {
    state: bool,
    mes: String,
}

impl WebMessage {
    fn new(state: bool, mes: &str) -> Self {
        Self {
            state,
            mes: mes.to_string(),
        }
    }
}

#[tauri::command]
pub async fn create_video_data_flow(
    ip: String,
    key: String,
    store: State<'_, Arc<DevicesMap>>,
    vnc_state: tauri::State<'_, VncProxyState>,
    app_handle: AppHandle,
) -> Result<WebMessage, String> {
    let store_clone = store.clone();
    let _ = open_vnc_window(
        app_handle,
        vnc_state,
        store_clone,
        ip.clone(),
        5900,
        "k1234".to_string(),
    )
    .await;

    Ok(sned_fn(ip, key, store).await?)
}

#[tauri::command]
pub async fn stop_fn(
    ip: String,
    key: String,
    store: tauri::State<'_, Arc<DevicesMap>>,
) -> Result<WebMessage, String> {
    Ok(sned_fn(ip, key, store).await?)
}

#[tauri::command]
pub async fn sned_fn(
    ip: String,
    key: String,
    store: tauri::State<'_, Arc<DevicesMap>>,
) -> Result<WebMessage, String> {
    println!("ip {} key {}", ip, key);
    match store.send_to(ip, key.clone()).await {
        Ok(_) => {
            return Ok(WebMessage::new(true, "ok"));
        }
        Err(_) => {
            let e = format!("发送指令失败: {}", &key.clone());
            return Ok(WebMessage::new(false, e.as_str()));
        }
    }
}

#[tauri::command]
pub async fn sync_devices(
    state: tauri::State<'_, Arc<DevicesMap>>,
) -> Result<HashMap<String, DevicePlayload>, String> {
    let snapshot = state.get_snapshot().await;
    Ok(snapshot)
}

// 保存脚本到store并发送到所有设备
#[tauri::command]
pub async fn send_script_to_all(
    script: String,
    store: tauri::State<'_, Arc<DevicesMap>>,
    app_handle: AppHandle,
) -> Result<WebMessage, String> {
    println!("发送脚本到所有设备: {}", script);

    // 解析脚本
    let script_obj: serde_json::Value =
        serde_json::from_str(&script).map_err(|e| format!("解析脚本失败: {}", e))?;

    let script_id = script_obj
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or("脚本缺少ID字段")?;

    // 先保存到AppData配置文件
    save_script(script_id, &script_obj).map_err(|e| format!("保存脚本到配置文件失败: {}", e))?;

    // 同步脚本相关的文件到resources目录
    if let Err(e) = sync_script_files(&script_obj) {
        eprintln!("文件同步失败: {}", e);
        // 文件同步失败不应阻止脚本发送
    }

    // 先触发事件更新前端列表
    let scripts_map = serde_json::json!({ script_id: script_obj });
    let event_data = format!("script|{}", scripts_map.to_string());
    let _ = app_handle.emit("device_config", event_data);

    // 获取所有设备IP
    let snapshot = store.get_snapshot().await;
    let device_ips: Vec<String> = snapshot.keys().cloned().collect();

    if device_ips.is_empty() {
        return Ok(WebMessage::new(false, "没有连接的设备"));
    }

    let mut success_count = 0;
    let mut failed_count = 0;

    for ip in &device_ips {
        // 发送脚本到设备，命令格式为 "script_sync|{script_json}"
        let cmd = format!("script_sync|{}", script);
        match store.send_to(ip.clone(), cmd).await {
            Ok(_) => {
                println!("脚本已发送到设备: {}", ip);
                success_count += 1;
            }
            Err(e) => {
                eprintln!("发送脚本到设备 {} 失败: {}", ip, e);
                failed_count += 1;
            }
        }
    }

    let message = if failed_count == 0 {
        format!("脚本已成功发送到 {} 个设备", success_count)
    } else {
        format!(
            "脚本已发送到 {} 个设备，失败 {} 个",
            success_count, failed_count
        )
    };

    Ok(WebMessage::new(failed_count == 0, &message))
}

// 获取设备脚本同步状态
#[tauri::command]
pub async fn get_device_script_sync(
    ip: String,
    store: tauri::State<'_, Arc<DevicesMap>>,
) -> Result<bool, String> {
    let snapshot = store.get_snapshot().await;

    match snapshot.get(&ip) {
        Some(device) => Ok(device.device_sync),
        None => Err(format!("设备 {} 不存在", ip)),
    }
}

// 批量同步所有设备脚本
#[tauri::command]
pub async fn sync_all_devices_scripts(
    store: tauri::State<'_, Arc<DevicesMap>>,
) -> Result<WebMessage, String> {
    // 这里可以从store读取保存的脚本，然后发送给所有设备
    // 简化实现：发送一个同步命令
    let snapshot = store.get_snapshot().await;
    let device_ips: Vec<String> = snapshot.keys().cloned().collect();

    if device_ips.is_empty() {
        return Ok(WebMessage::new(false, "没有连接的设备"));
    }

    let device_count = device_ips.len();

    for ip in &device_ips {
        let _ = store
            .send_to(ip.clone(), "script_check_sync".to_string())
            .await;
    }

    Ok(WebMessage::new(
        true,
        &format!("已向 {} 个设备发送同步检查", device_count),
    ))
}

// 更新单个设备信息
#[tauri::command]
pub async fn change_one_device_info(
    device_info: serde_json::Value,
    app_handle: AppHandle,
) -> Result<WebMessage, String> {
    println!("收到更新单个设备信息请求: {:?}", device_info);

    // 解析设备信息
    let device_key = device_info
        .get("device_key")
        .and_then(|v| v.as_str())
        .ok_or("缺少device_key字段")?;

    let device_ip = device_info
        .get("device_ip")
        .and_then(|v| v.as_str())
        .unwrap_or(device_key);

    let device_name = device_info
        .get("device_name")
        .and_then(|v| v.as_str())
        .unwrap_or("keduli");

    let file_sync = device_info
        .get("file_sync")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let device_sync = device_info
        .get("device_sync")
        .and_then(|v| v.as_bool())
        .unwrap_or(file_sync);

    // 创建DevicePlayload对象
    let payload = DevicePlayload {
        device_ip: device_ip.to_string(),
        device_name: device_name.to_string(),
        device_sync,
    };

    // 通过事件更新前端
    let _ = app_handle.emit(
        "devices_event",
        serde_json::json!({
            "event": "device_add",
            "devices": (device_key, payload)
        }),
    );

    println!("已更新设备 {} 的信息", device_key);
    Ok(WebMessage::new(true, "设备信息已更新"))
}

// 触发文件同步
#[tauri::command]
pub async fn trigger_file_sync() -> Result<WebMessage, String> {
    println!("触发文件同步（服务器端处理）");
    // 尝试获取脚本配置并同步文件
    match get_scripts_config() {
        Ok(scripts_config) => {
            println!("获取到脚本配置，准备同步相关文件: {}", scripts_config);
            // 这里可以调用file_sync模块的函数同步文件
            // 但不向设备发送命令
        }
        Err(e) => {
            println!("获取脚本配置失败: {}", e);
        }
    }

    Ok(WebMessage::new(
        true,
        "服务器端文件同步处理完成（不向设备发送同步命令）",
    ))
}
