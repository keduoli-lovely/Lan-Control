use tauri::Emitter;

use crate::client::{
    screenshot::get_screenshot::ScreenshotStore,
    server_info::{server_ip, ServerInfo},
    ws::{command_ws::CommandClient, screenshots_ws::ScreenshotsSender},
};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};

pub struct AppStore {
    pub screenshot_task: Mutex<Option<Arc<ScreenshotStore>>>,
    pub server_info: Arc<Mutex<ServerInfo>>,
    pub ws_reconnect_count: AtomicUsize,
    pub reconnect_count_max: AtomicUsize,
    pub exit_flag: AtomicBool,
    pub command_client: Mutex<Option<Arc<CommandClient>>>,
    pub screenshots_sender: Mutex<Option<Arc<ScreenshotsSender>>>,
    pub reconnect_interval: AtomicUsize, // 重连间隔（秒）
    pub reconnect_ws: AtomicBool,        // 是否启用重连
}
impl AppStore {
    pub fn new() -> Self {
        Self {
            screenshot_task: Mutex::new(None),
            server_info: Arc::new(Mutex::new(ServerInfo::new("127.0.0.1".to_string(), 9000))),
            ws_reconnect_count: AtomicUsize::new(0),
            reconnect_count_max: AtomicUsize::new(20),
            exit_flag: AtomicBool::new(false),
            command_client: Mutex::new(None),
            screenshots_sender: Mutex::new(None),
            reconnect_interval: AtomicUsize::new(2), // 默认2秒
            reconnect_ws: AtomicBool::new(true),     // 默认启用重连
        }
    }

    pub fn get_exit_flag(&self) -> bool {
        self.exit_flag.load(Ordering::Relaxed)
    }

    pub fn get_reconnect_interval(&self) -> usize {
        self.reconnect_interval.load(Ordering::Relaxed)
    }

    pub fn get_reconnect_ws(&self) -> bool {
        self.reconnect_ws.load(Ordering::Relaxed)
    }
}

#[tauri::command]
pub async fn initialize_runtime(
    store: tauri::State<'_, Arc<AppStore>>,
    app_handle: tauri::AppHandle,
) -> Result<bool, String> {
    let ip = server_ip(app_handle.clone(), store.inner().clone())
        .await
        .unwrap();
    match ip {
        Some((
            ip,
            port,
            out_max_num,
            ws_command_path,
            ws_screenshot_path,
            screenshot_interval_str,
            auto_start_str,
            broadcast_port_str,
            preview_str,
            pause_preview_str,
            reconnect_ws_str,
            reconnect_interval_str,
        )) => {
            let port_num = port.parse::<usize>().unwrap_or(9000);
            let screenshot_interval = screenshot_interval_str.parse::<u64>().unwrap_or(500);
            let auto_start = auto_start_str.parse::<bool>().unwrap_or(false);
            let broadcast_port = broadcast_port_str.parse::<u16>().unwrap_or(13140);
            let preview = preview_str.parse::<bool>().unwrap_or(true);
            let pause_preview = pause_preview_str.parse::<bool>().unwrap_or(false);
            let reconnect_ws = reconnect_ws_str.parse::<bool>().unwrap_or(true);
            let reconnect_interval = reconnect_interval_str.parse::<u8>().unwrap_or(2);

            // 更新服务器信息
            {
                let mut info = store.server_info.lock().unwrap();
                info.ip = ip.clone();
                info.port = port_num;
                info.ws_command_path = ws_command_path.clone();
                info.ws_screenshot_path = ws_screenshot_path.clone();
                info.screenshot_interval = screenshot_interval;
                info.auto_start = auto_start;
                info.broadcast_port = broadcast_port;
                info.preview = preview;
                info.pause_preview = pause_preview;
                info.reconnect_ws = reconnect_ws;
                info.reconnect_interval = reconnect_interval;
            }

            // 设置重连次数限制
            {
                store.reconnect_count_max.store(
                    out_max_num.parse::<usize>().unwrap_or(20),
                    Ordering::Relaxed,
                );
            }

            // 设置重连间隔和重连开关
            {
                store
                    .reconnect_interval
                    .store(reconnect_interval as usize, Ordering::Relaxed);
                store.reconnect_ws.store(reconnect_ws, Ordering::Relaxed);
                println!(
                    "配置参数已应用 - 重连间隔: {}秒, 启用重连: {}",
                    reconnect_interval, reconnect_ws
                );
            }

            // 构建WebSocket URL并启动CommandClient
            let ws_command_url = format!("ws://{}:{}{}", ip, port_num, ws_command_path);
            println!("Connecting to command WebSocket: {}", ws_command_url);

            // 启动CommandClient - 克隆store以避免移动问题
            let store_clone = store.inner().clone();
            let app_handle_clone = app_handle.clone();
            let command_client =
                CommandClient::start(&ws_command_url, store_clone.clone(), app_handle_clone);

            // 保存CommandClient到store
            {
                let mut client_guard = store_clone.command_client.lock().unwrap();
                *client_guard = Some(command_client);
            }

            // 构建截图WebSocket URL并启动ScreenshotsSender
            let ws_screenshot_url = format!("ws://{}:{}{}", ip, port_num, ws_screenshot_path);
            println!("Connecting to screenshot WebSocket: {}", ws_screenshot_url);

            let app_handle_clone2 = app_handle.clone();
            let screenshots_sender = ScreenshotsSender::start(
                &ws_screenshot_url,
                store_clone.clone(),
                app_handle_clone2,
            );

            // 在保存到store之前获取sender
            let screenshot_sender = screenshots_sender.sender();

            // 保存ScreenshotsSender到store
            {
                let mut sender_guard = store_clone.screenshots_sender.lock().unwrap();
                *sender_guard = Some(screenshots_sender);
            }

            // 创建并启动截图任务，使用从广播接收的截图频率和预览配置
            {
                // 如果preview为false，则禁用截图（间隔设为0）
                let effective_interval = if !preview { 0 } else { screenshot_interval };

                let screenshot_task =
                    ScreenshotStore::start(0, effective_interval, screenshot_sender);
                let mut task_guard = store_clone.screenshot_task.lock().unwrap();
                *task_guard = Some(screenshot_task);

                if !preview {
                    println!("预览已禁用，截图器已暂停");
                } else {
                    println!("截图任务已启动，截图频率: {}ms", screenshot_interval);
                }
            }
            // 设置开机自启
            let auto_run = match store.server_info.lock() {
                Ok(info) => info.auto_start,
                Err(_) => false,
            };
            let _ = app_handle.emit("command", format!("auto_run_state|{}", auto_run));
            return Ok(true);
        }
        None => {
            return Ok(false);
        }
    }
}

// 关闭所有WebSocket连接
#[tauri::command]
pub fn close_all_ws(store: tauri::State<'_, Arc<AppStore>>) -> Result<(), String> {
    println!("Closing all WebSocket connections...");

    // 设置退出标志
    store.exit_flag.store(true, Ordering::Relaxed);

    // 关闭CommandClient
    {
        let mut client_guard = store.command_client.lock().unwrap();
        if let Some(client) = client_guard.take() {
            client.exit();
            println!("CommandClient closed");
        }
    }

    // 关闭ScreenshotsSender
    {
        let mut sender_guard = store.screenshots_sender.lock().unwrap();
        if let Some(sender) = sender_guard.take() {
            sender.exit();
            println!("ScreenshotsSender closed");
        }
    }

    // 重置重连计数
    store.ws_reconnect_count.store(0, Ordering::Relaxed);
    println!("All WebSocket connections closed");

    Ok(())
}
