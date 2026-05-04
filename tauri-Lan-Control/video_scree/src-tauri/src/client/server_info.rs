use std::sync::{atomic::Ordering, Arc};

use serde_json;
use tokio::net::UdpSocket;

use crate::client::initialize::AppStore;

pub struct ServerInfo {
    pub ip: String,
    pub port: usize,
    pub ws_command_path: String,
    pub ws_screenshot_path: String,
    pub screenshot_interval: u64,
    pub auto_start: bool,
    pub broadcast_port: u16,
    pub preview: bool,
    pub pause_preview: bool,
    pub reconnect_ws: bool,
    pub reconnect_interval: u8,
}

impl ServerInfo {
    pub fn new(ip: String, port: usize) -> Self {
        Self {
            ip,
            port,
            ws_command_path: "/ws/command".to_string(),
            ws_screenshot_path: "/ws/pic".to_string(),
            screenshot_interval: 500, // 默认500ms
            auto_start: false,
            broadcast_port: 13140,
            preview: true,
            pause_preview: false,
            reconnect_ws: true,
            reconnect_interval: 2,
        }
    }
}

pub async fn server_ip(
    _app_handle: tauri::AppHandle,
    store: Arc<AppStore>,
) -> std::io::Result<
    Option<(
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
    )>,
> {
    let res = UdpSocket::bind("0.0.0.0:13140").await?;
    let mut buf = [0u8; 1024];
    loop {
        println!("ad run");
        let (len, addr) = res.recv_from(&mut buf).await?;
        if !addr.is_ipv4() {
            return Ok(None);
        }

        let ip = addr
            .to_string()
            .split(":")
            .next()
            .unwrap_or("0")
            .to_string();
        if ip == "0" {
            return Ok(None);
        }

        let msg = std::str::from_utf8(&buf[..len]).unwrap_or("");
        let json = serde_json::from_str::<serde_json::Value>(msg).expect("获取广播数据失败");

        // 解析所有配置参数
        let ws_command_path = json["ws_command_path"]
            .as_str()
            .unwrap_or("/ws/command")
            .to_string();

        let ws_screenshot_path = json["ws_screenshot_path"]
            .as_str()
            .unwrap_or("/ws/pic")
            .to_string();

        let screenshot_interval = json["screenshot_interval"]
            .as_str()
            .unwrap_or("500")
            .to_string();

        let auto_start = json["auto_start"].as_bool().unwrap_or(false).to_string();

        let broadcast_port = json["broadcast_port"].as_u64().unwrap_or(13140).to_string();

        let preview = json["preview"].as_bool().unwrap_or(true).to_string();

        let pause_preview = json["pause_preview"].as_bool().unwrap_or(false).to_string();

        let reconnect_ws = json["reconnect_ws"].as_bool().unwrap_or(true).to_string();

        let reconnect_interval = json["reconnect_interval"].as_u64().unwrap_or(2).to_string();

        // 重新启动所有项目 - 如果exit_flag为true，重置它并返回None让前端重试
        if store.get_exit_flag() {
            println!("检测到重启标志，重置计数器并返回None触发前端重试...");
            store.ws_reconnect_count.store(0, Ordering::Relaxed);
            store.exit_flag.store(false, Ordering::Relaxed);
            return Ok(None);
        }

        return Ok(Some((
            ip,
            json["port"].to_string(),
            json["outMaxNum"].to_string(),
            ws_command_path,
            ws_screenshot_path,
            screenshot_interval,
            auto_start,
            broadcast_port,
            preview,
            pause_preview,
            reconnect_ws,
            reconnect_interval,
        )));
    }
}
