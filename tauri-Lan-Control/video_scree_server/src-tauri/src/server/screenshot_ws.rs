use crate::server::ws_server::SetState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo, State,
    },
    response::IntoResponse,
};
use base64::{engine::general_purpose, Engine as _};
use futures_util::StreamExt;
use std::sync::Arc;
use std::net::SocketAddr;
use tauri::Emitter;

pub async fn screenshot_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<SetState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        let ip = addr.ip().to_string();
        handle_screenshot_connection(socket, state.app_handle.clone(), ip).await;
    })
}

async fn handle_screenshot_connection(socket: WebSocket, app_handle: tauri::AppHandle, device_ip: String) {
    let (_, mut ws_read) = socket.split();

    while let Some(Ok(msg)) = ws_read.next().await {
        match msg {
            Message::Binary(bytes) => {
                let base64_pic = general_purpose::STANDARD.encode(bytes);
                let data_url = format!("data:image/jpeg;base64,{}", base64_pic);
                let _ = app_handle.emit("device_preview", (device_ip.clone(), data_url));
            }
            _ => {}
        }
    }
    
    // 设备断开连接时，发送清除预览的事件
    let _ = app_handle.emit("device_preview_clear", device_ip);
}