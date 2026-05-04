pub mod vnc_proxy;
pub mod commadn_ws;
use commadn_ws::command_handler;

use crate::server::{
    handle_run::devices_map::DevicesMap,
    screenshot_ws::screenshot_handler,
    
};
use axum::{routing::get, Router};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tauri::Manager;
use tokio::net::TcpListener;

// 设备信息
#[derive(Deserialize, Debug, Clone)]
pub struct DeviceInfoData {
    pub device_key: String,
    pub device_name: String,
    pub device_ip: String,
    pub file_sync: bool,
}
#[derive(Clone)]
pub struct SetState {
    pub store: Arc<DevicesMap>,
    pub app_handle: tauri::AppHandle,
}

pub fn start_background_tasks(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let store_state = app_handle.state::<Arc<DevicesMap>>();
        let store: Arc<DevicesMap> = store_state.inner().clone();

        let state = Arc::new(SetState {
            store: store.clone(),
            app_handle: app_handle.clone(),
        });

        let app = Router::new()
            .route("/ws/pic", get(screenshot_handler).with_state(state.clone()))
            .route(
                "/ws/command",
                get(command_handler).with_state(state.clone()),
            );
        let listener = TcpListener::bind("0.0.0.0:9000").await.unwrap();
        println!("服务启动: ws://0.0.0.0:9000");

        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
}