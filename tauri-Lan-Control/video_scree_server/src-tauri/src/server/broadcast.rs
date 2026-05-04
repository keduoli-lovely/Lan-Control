use serde_json::json;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::sleep;

pub async fn broadcast_fn(app_handle: tauri::AppHandle) -> std::io::Result<()> {
    let res = UdpSocket::bind("0.0.0.0:0").await?;
    res.set_broadcast(true)?;

    loop {
        // 从应用配置读取设置
        let config = get_config_from_store(&app_handle).await;

        // 构建广播数据，包含所有配置参数
        let payload = json!({
            "port": config.ws_port,
            "outMaxNum": config.reconnect_times,
            "ws_command_path": config.ws_command_path,
            "ws_screenshot_path": config.ws_screenshot_path,
            "screenshot_interval": config.screenshot_interval,
            "auto_start": config.auto_start,
            "broadcast_port": config.broadcast_port,
            "preview": config.preview,
            "pause_preview": config.pause_preview,
            "reconnect_ws": config.reconnect_ws,
            "reconnect_interval": config.reconnect_interval
        });

        let broadcast_addr = format!("255.255.255.255:{}", config.broadcast_port);

        if let Err(e) = res
            .send_to(payload.to_string().as_bytes(), broadcast_addr)
            .await
        {
            eprintln!("广播发送失败: {}", e);
        }

        sleep(Duration::from_millis(5000)).await;
    }
}

#[derive(Default, Clone)]
struct BroadcastConfig {
    ws_port: u16,
    broadcast_port: u16,
    reconnect_times: u8,
    ws_command_path: String,
    ws_screenshot_path: String,
    screenshot_interval: u64,
    auto_start: bool,
    preview: bool,
    pause_preview: bool,
    reconnect_ws: bool,
    reconnect_interval: u8,
}

async fn get_config_from_store(app_handle: &tauri::AppHandle) -> BroadcastConfig {
    use tauri_plugin_store::StoreBuilder;

    // 尝试从Store加载配置
    let store_result = StoreBuilder::new(app_handle, ".server_settings.json").build();

    match store_result {
        Ok(store) => {
            // 尝试加载配置，reload()可能不是async的
            if let Err(e) = store.reload() {
                eprintln!("加载Store失败: {}", e);
                // 返回默认配置
                return BroadcastConfig {
                    ws_port: 9000,
                    broadcast_port: 13140,
                    reconnect_times: 5,
                    ws_command_path: "/ws/command".to_string(),
                    ws_screenshot_path: "/ws/pic".to_string(),
                    screenshot_interval: 500,
                    auto_start: false,
                    preview: true,
                    pause_preview: false,
                    reconnect_ws: true,
                    reconnect_interval: 2,
                };
            }

            // 读取配置字段
            let config_value: serde_json::Value = store
                .get("config")
                .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));

            // 解析配置，使用默认值
            BroadcastConfig {
                ws_port: config_value
                    .get("wsPort")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u16)
                    .unwrap_or(9000),
                broadcast_port: config_value
                    .get("broadcastPort")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u16)
                    .unwrap_or(13140),
                reconnect_times: config_value
                    .get("reconnectTimes")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u8)
                    .unwrap_or(5),
                ws_command_path: config_value
                    .get("wsCommandPath")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "/ws/command".to_string()),
                ws_screenshot_path: config_value
                    .get("wsScreenshotPath")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "/ws/pic".to_string()),
                screenshot_interval: config_value
                    .get("screenshotInterval")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u64)
                    .unwrap_or(500),
                auto_start: config_value
                    .get("autoStart")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                preview: config_value
                    .get("preview")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
                pause_preview: config_value
                    .get("pausePreview")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                reconnect_ws: config_value
                    .get("reconnectWs")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
                reconnect_interval: config_value
                    .get("reconnectInterval")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u8)
                    .unwrap_or(2),
            }
        }
        Err(e) => {
            eprintln!("创建Store失败: {}", e);
            // 返回默认配置
            BroadcastConfig {
                ws_port: 9000,
                broadcast_port: 13140,
                reconnect_times: 5,
                ws_command_path: "/ws/command".to_string(),
                ws_screenshot_path: "/ws/pic".to_string(),
                screenshot_interval: 500,
                auto_start: false,
                preview: true,
                pause_preview: false,
                reconnect_ws: true,
                reconnect_interval: 2,
            }
        }
    }
}
