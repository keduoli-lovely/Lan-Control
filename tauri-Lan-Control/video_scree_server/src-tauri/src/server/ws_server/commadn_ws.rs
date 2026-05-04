use crate::server::ws_server::SetState;
use crate::server::file_monitor::get_scripts_config;
use axum::{
    extract::{
        ws::{Message, WebSocketUpgrade},
        ConnectInfo, State,
    },
    response::IntoResponse,
};
use futures_util::StreamExt;
use std::net::SocketAddr;
use std::sync::Arc;
use tauri::Emitter;

pub async fn command_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<SetState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        let (ws_write, mut ws_read) = socket.split();
        let ip = addr.ip().to_string();
        let app_handle = state.app_handle.clone();
        let store = state.store.clone();

        // 添加设备
        store
            .add_device(ip.clone(), ws_write, app_handle.clone())
            .await
            .unwrap();
        // 发送消息
        store
            .send_to(ip.clone(), format!("key|{}", ip))
            .await
            .unwrap();
        // 监听消息
        while let Some(msg) = ws_read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    println!("message {}", text);
                    let _ = app_handle.emit("device_config", text.to_string());
                    
                    // 处理设备信息消息
                    if text.starts_with("device_info|") {
                        let info_json = text.trim_start_matches("device_info|");
                        println!("收到设备信息: {}", info_json);
                        
                        match serde_json::from_str::<crate::server::ws_server::DeviceInfoData>(info_json) {
                            Ok(mut device_info) => {
                                // 在更新设备信息前，检查当前设备的同步状态
                                let snapshot = store.get_snapshot().await;
                                if let Some(existing_device) = snapshot.get(&device_info.device_key) {
                                    // 如果现有设备已经有同步状态，保留它
                                    // 除非新消息明确设置了file_sync字段
                                    // 注意：device_info中的file_sync可能来自客户端旧数据
                                    // 所以不在这里更新file_sync，只在device_sync消息中更新
                                    device_info.file_sync = existing_device.device_sync;
                                    println!("从现有设备保持同步状态: {}", existing_device.device_sync);
                                } else {
                                    println!("新设备，使用默认同步状态: {}", device_info.file_sync);
                                }
                                
                                // 更新设备信息 - 传递引用
                                let _ = store.update_device(&device_info, app_handle.clone()).await;
                                println!("设备信息已更新: key={}, name={}, ip={}, sync={}", 
                                    device_info.device_key, device_info.device_name, device_info.device_ip, device_info.file_sync);
                            }
                            Err(e) => {
                                eprintln!("解析设备信息失败: {}", e);
                            }
                        }
                    }
                    // 处理设备同步状态消息
                    else if text.starts_with("device_sync|") {
                        let sync_value = text.trim_start_matches("device_sync|");
                        let file_sync = sync_value == "true";
                        
                        // 更新设备同步状态 - 使用特殊标记表示只更新同步状态，不更新名称
                        let device_info = crate::server::ws_server::DeviceInfoData {
                            device_key: ip.clone(),
                            device_name: "SYNC_STATUS_UPDATE".to_string(), // 特殊标记，表示只更新同步状态
                            device_ip: ip.clone(),
                            file_sync,
                        };
                        
                        // 调用update_device更新设备信息
                        let _ = store.update_device(&device_info, app_handle.clone()).await;
                        println!("设备 {} 同步状态更新为: {}", ip, file_sync);
                        
                        // 发送包含IP的同步状态消息到前端，格式为 "device_sync_ip|{ip}|{status}"
                        let sync_message = format!("device_sync_ip|{}|{}", ip, file_sync);
                        let _ = app_handle.emit("device_config", sync_message);
                    }
                    // 处理get_script请求
                    else if text.starts_with("get_script") {
                        println!("收到get_script请求，准备发送脚本给设备: {}", ip);
                        
                        // 从配置文件读取脚本配置
                        match get_scripts_config() {
                            Ok(scripts_config) => {
                                let scripts_str = serde_json::to_string(&scripts_config)
                                    .unwrap_or_else(|e| {
                                        eprintln!("序列化脚本配置失败: {}", e);
                                        "{}".to_string()
                                    });
                                
                                // 如果脚本配置为空，不发送任何消息
                                if scripts_str != "{}" && scripts_str != "null" {
                                    // 逐个发送脚本给客户端
                                    if let Some(scripts_obj) = scripts_config.as_object() {
                                        for (script_id, script) in scripts_obj {
                                            let script_json = serde_json::to_string(script)
                                                .unwrap_or_else(|e| {
                                                    eprintln!("序列化脚本{}失败: {}", script_id, e);
                                                    "{}".to_string()
                                                });
                                            
                                            if script_json != "{}" {
                                                let cmd = format!("script_sync|{}", script_json);
                                                println!("发送脚本 {} 到设备 {}", script_id, ip);
                                                if let Err(e) = store.send_to(ip.clone(), cmd).await {
                                                    eprintln!("发送脚本到设备失败: {}", e);
                                                } else {
                                                    println!("脚本 {} 已发送到设备 {}", script_id, ip);
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    println!("服务器脚本配置为空，不发送脚本");
                                }
                            }
                            Err(e) => {
                                eprintln!("获取脚本配置失败: {}", e);
                            }
                        }
                    }
                    // 处理request_file_sync请求
                    else if text.starts_with("request_file_sync|") {
                        println!("收到文件同步请求: {}", text);
                        
                        // 解析缺失的文件列表
                        let missing_files_str = text.trim_start_matches("request_file_sync|");
                        let missing_files: Vec<&str> = missing_files_str.split(';').collect();
                        
                        println!("需要同步的文件: {:?}", missing_files);
                        
                        // 从配置文件读取脚本配置，以获取文件路径
                        match get_scripts_config() {
                            Ok(scripts_config) => {
                                if let Some(scripts_obj) = scripts_config.as_object() {
                                    for missing_file in missing_files {
                                        // 支持两种格式：
                                        // 1. 旧格式: "脚本ID:文件路径" (如 "vnc:ultravnc_x64")
                                        // 2. 新格式: "脚本ID" (如 "vnc" 或 "Bat_To_Exe_Converter")
                                        
                                        if missing_file.contains(':') {
                                            // 格式1: "脚本ID:文件路径"
                                            let parts: Vec<&str> = missing_file.splitn(2, ':').collect();
                                            let script_id = parts[0].trim();
                                            let file_path = parts[1].trim();
                                            
                                            // 查找对应的脚本
                                            if let Some(script) = scripts_obj.get(script_id) {
                                                // 根据脚本类型同步文件
                                                sync_files_for_script(script, file_path, &ip, &store).await;
                                            } else {
                                                println!("未找到脚本: {}", script_id);
                                            }
                                        } else {
                                            // 格式2: 只有脚本ID
                                            let script_id = missing_file.trim();
                                            
                                            // 查找对应的脚本
                                            if let Some(script) = scripts_obj.get(script_id) {
                                                // 从脚本配置中获取文件路径
                                                let script_path = script.get("path")
                                                    .and_then(|v| v.as_str())
                                                    .unwrap_or("");
                                                
                                                if !script_path.is_empty() {
                                                    // 根据脚本类型同步文件
                                                    sync_files_for_script(script, script_path, &ip, &store).await;
                                                } else {
                                                    println!("脚本 {} 路径为空", script_id);
                                                }
                                            } else {
                                                println!("未找到脚本: {}", script_id);
                                            }
                                        }
                                    }
                                    
                                    // 所有文件同步完成后，发送同步成功状态给设备
                                    // 延迟一段时间确保文件已同步完成
                                    let ip_clone = ip.clone();
                                    let store_clone = store.clone();
                                    tokio::spawn(async move {
                                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                                        if let Err(e) = store_clone.send_to(ip_clone.clone(), "script_check_sync".to_string()).await {
                                            eprintln!("发送脚本检查同步命令失败: {}", e);
                                        } else {
                                            println!("已发送script_check_sync命令给设备: {}", ip_clone);
                                        }
                                    });
                                }
                            }
                            Err(e) => {
                                eprintln!("获取脚本配置失败: {}", e);
                            }
                        }
                    }
                    // 处理VNC下载状态消息
                    else if text.starts_with("vnc_download|") {
                        println!("收到VNC下载状态消息: {}", text);
                        
                        // 将VNC下载状态消息转发到前端
                        let vnc_status_message = text.trim_start_matches("vnc_download|");
                        let formatted_message = format!("vnc_download|{}|{}", ip, vnc_status_message);
                        let formatted_message_clone = formatted_message.clone();
                        let _ = app_handle.emit("device_config", formatted_message);
                        println!("已转发VNC下载状态到前端: {}", formatted_message_clone);
                    }
                    // 处理VNC启动状态消息
                    else if text.starts_with("vnc_started|") {
                        println!("收到VNC启动状态消息: {}", text);
                        
                        // 将VNC启动状态消息转发到前端
                        let vnc_started_message = text.trim_start_matches("vnc_started|");
                        let formatted_message = format!("vnc_started|{}|{}", ip, vnc_started_message);
                        let formatted_message_clone = formatted_message.clone();
                        let _ = app_handle.emit("device_config", formatted_message);
                        println!("已转发VNC启动状态到前端: {}", formatted_message_clone);
                    }
                }
                Ok(Message::Close(_)) => {
                    // 设备断开
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!("error listener text {:?}", e);
                    break;
                }
            }
        }
        // 移除设备
        store.remove_device(&ip, app_handle.clone()).await.unwrap();
        println!("客户端 {} 断开链接 -- media", ip);
    })
}

// 同步脚本相关的文件到设备
async fn sync_files_for_script(
    script: &serde_json::Value,
    file_path: &str,
    ip: &str,
    store: &crate::server::ws_server::DevicesMap,
) {
    println!("同步脚本文件: {}, 文件路径: {}", 
        script.get("id").and_then(|v| v.as_str()).unwrap_or("unknown"), 
        file_path);
    
    // 获取脚本类型和路径
    let path_type = script.get("pathType")
        .and_then(|v| v.as_str())
        .unwrap_or("file");
    let script_path = script.get("path")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    if script_path.is_empty() {
        println!("脚本路径为空，无法同步");
        return;
    }
    
    match path_type {
        "file" => {
            // 单个文件同步
            let source_path = std::path::Path::new(script_path);
            if source_path.exists() {
                // 读取文件并转换为base64
                match crate::server::file_sync::read_file_as_base64(source_path) {
                    Ok(base64_content) => {
                        // 获取文件名
                        let file_name = source_path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");
                        
                        // 发送文件同步命令
                        let cmd = format!("file_sync|{}|{}", file_name, base64_content);
                        if let Err(e) = store.send_to(ip.to_string(), cmd).await {
                            eprintln!("发送文件同步失败: {}", e);
                        } else {
                            println!("文件同步成功: {}", file_name);
                        }
                    }
                    Err(e) => {
                        eprintln!("读取文件失败: {}", e);
                    }
                }
            } else {
                println!("源文件不存在: {}", script_path);
            }
        }
        "folder" => {
            // 文件夹类型，需要同步整个文件夹的所有文件
            let folder_path = std::path::Path::new(script_path);
            if !folder_path.exists() || !folder_path.is_dir() {
                println!("文件夹不存在或不是目录: {}", script_path);
                return;
            }
            
            // 获取文件夹名
            let folder_name = folder_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            
            // 遍历文件夹中的所有文件
            match std::fs::read_dir(folder_path) {
                Ok(entries) => {
                    let mut files_synced = 0;
                    let mut errors = 0;
                    
                    for entry in entries.filter_map(|e| e.ok()) {
                        let entry_path = entry.path();
                        if entry_path.is_file() {
                            // 读取文件并转换为base64
                            match crate::server::file_sync::read_file_as_base64(&entry_path) {
                                Ok(base64_content) => {
                                    // 构建相对于文件夹的相对路径
                                    if let Ok(relative_path) = entry_path.strip_prefix(folder_path) {
                                        if let Some(relative_str) = relative_path.to_str() {
                                            // 确保路径使用正斜杠分隔符，兼容Windows/Linux
                                            let normalized_path = relative_str.replace('\\', "/");
                                            // 添加文件夹名前缀：folder_name/relative_path
                                            let full_path = format!("{}/{}", folder_name, normalized_path);
                                            let cmd = format!("file_sync|{}|{}", full_path, base64_content);
                                            if let Err(e) = store.send_to(ip.to_string(), cmd).await {
                                                eprintln!("发送文件同步失败: {}", e);
                                                errors += 1;
                                            } else {
                                                println!("文件夹文件同步成功: {}", full_path);
                                                files_synced += 1;
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("读取文件失败: {}", e);
                                    errors += 1;
                                }
                            }
                        }
                    }
                    
                    println!("文件夹类型脚本同步完成: 成功同步{}个文件，失败{}个文件", files_synced, errors);
                    
                    if files_synced == 0 && errors == 0 {
                        println!("文件夹为空，没有文件需要同步");
                    }
                }
                Err(e) => {
                    eprintln!("读取文件夹失败: {}", e);
                }
            }
        }
        _ => {
            println!("不支持的脚本类型: {}", path_type);
        }
    }
}
