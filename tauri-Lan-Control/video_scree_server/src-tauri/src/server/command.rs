use std::sync::{Arc, Mutex};
use tauri::async_runtime::JoinHandle;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use crate::server::handle_run::devices_map::DevicesMap;

#[derive(Clone)]
pub struct VncProxyState {
    proxies: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl VncProxyState {
    pub fn new() -> Self {
        Self {
            proxies: Arc::new(Mutex::new(None)),
        }
    }
}

// 创建并连接 vnc
#[tauri::command]
pub async fn open_vnc_window(
    app: tauri::AppHandle,
    state: tauri::State<'_, VncProxyState>,
    store: tauri::State<'_, Arc<DevicesMap>>,
    host: String,
    port: u32,
    pwd: String,
) -> Result<(), String> {
    if host.is_empty() || pwd.is_empty() {
        return Ok(());
    }

    let label = "vnc-window";
    // 检查窗口是否已存在
    if app.get_webview_window(label).is_some() {
        println!("window already exists");
        return Ok(());
    }

    // 启动 VNC 代理，将前端传入的 host:port 作为目标
    let target = format!("{}:{}", host, port);
    let proxy_handle = tauri::async_runtime::spawn(async move {
        let _ = crate::server::ws_server::vnc_proxy::start_vnc_proxy("127.0.0.1:5901", &target).await;
    });

    {
        // 保存vnc 代理
        if let Ok(mut res) = state.proxies.lock() {
            *res = Some(proxy_handle)
        }
    }

    let url = format!("/vnc-window.html?host=127.0.0.1&port=5901&password={}", pwd);
    let state_handle = state.inner().clone();
    let store_clone = store.inner().clone();
    let host_clone = host.clone();

    // 创建窗口
    let win = WebviewWindowBuilder::new(&app, label, WebviewUrl::App(url.into()))
        .title("vnc-window")
        .inner_size(1200.0, 800.0)
        .resizable(true)
        .build()
        .map_err(|e| e.to_string())?;
    win.on_window_event(move |event| match event {
        tauri::WindowEvent::CloseRequested { .. } => {
            // 中止VNC代理
            if let Ok(mut res) = state_handle.proxies.lock() {
                if let Some(handle) = res.take() {
                    handle.abort();
                }
            }
            
            // 发送恢复截图命令给客户端
            let store = store_clone.clone();
            let host = host_clone.clone();
            tauri::async_runtime::spawn(async move {
                match store.send_to(host.clone(), "resume_screenshot".to_string()).await {
                    Ok(_) => println!("已发送恢复截图命令到设备: {}", host),
                    Err(e) => eprintln!("发送恢复截图命令失败: {}", e),
                }
            });
        }
        _ => {}
    });

    Ok(())
}

// 测试命令：返回当前时间戳和服务器状态
#[tauri::command]
pub async fn test_server_status() -> Result<String, String> {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?;
    
    let status = format!(
        "Server is running - Timestamp: {}ms, Version: 1.0.0",
        now.as_millis()
    );
    
    Ok(status)
}

// 测试命令：运行系统命令并返回结果
#[tauri::command]
pub async fn run_system_command(cmd: String) -> Result<String, String> {
    use std::process::Command;
    
    println!("Running system command: {}", cmd);
    
    let output = Command::new("cmd")
        .args(&["/C", &cmd])
        .output()
        .map_err(|e| e.to_string())?;
    
    let result = if output.status.success() {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        format!("Command failed: {}", error_msg)
    };
    
    Ok(result)
}


