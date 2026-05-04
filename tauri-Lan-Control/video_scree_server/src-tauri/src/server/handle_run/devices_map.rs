use chrono::Local;
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use serde::Serialize;
use std::{collections::HashMap, sync::Arc};
use tauri::AppHandle;
use tauri::Emitter;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};
use axum::extract::ws::{Message, WebSocket};
use crate::server::ws_server::DeviceInfoData;
type Tx = SplitSink<WebSocket, Message>;
#[derive(Debug)]
pub struct DeviceInfo {
    ws: Tx,
    device_name: String,
    device_ip: String,
    device_sync: bool,
}

#[derive(Clone, Serialize)]
pub struct DevicePlayload {
    pub device_name: String,
    pub device_ip: String,
    pub device_sync: bool,
}

impl DeviceInfo {
    fn new(ws: Tx, ip: String) -> Self {
        Self {
            ws,
            device_name: "keduli".to_string(),
            device_ip: ip,
            device_sync: false,
        }
    }
}

#[allow(dead_code)]
pub struct DevicesMap {
    clients: Arc<Mutex<HashMap<String, DeviceInfo>>>,
    snapshot_task: Arc<Mutex<Option<JoinHandle<()>>>>,
    pub default_name: Arc<Mutex<String>>,
    pub snapshot_interval_time: Arc<Mutex<u64>>,
}

impl DevicesMap {
    pub fn new() -> Self {
        let ts = Local::now().format("%Y%m%d%H%M%S").to_string();
        let name = format!("keduoli-{}", ts);
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            snapshot_task: Arc::new(Mutex::new(None)),
            default_name: Arc::new(Mutex::new(name)),
            snapshot_interval_time: Arc::new(Mutex::new(60)),
        }
    }

    // 发送指定到客户端
    pub async fn send_to(&self, addr: String, msg: String) -> anyhow::Result<()> {
        let mut map = self.clients.lock().await;
        if let Some(tx) = map.get_mut(&addr) {
            tx.ws.send(Message::Text(msg.into())).await?;
        }
        Ok(())
    }

    // 管理设备 / 发送到前端
    // 添加
    pub async fn add_device(
        &self,
        ip: String,
        tx: Tx,
        app_handle: tauri::AppHandle,
    ) -> anyhow::Result<()> {
        let mut map = self.clients.lock().await;
        if !map.contains_key(&ip) {
            let device_info = DeviceInfo::new(tx, ip.clone());
            let device_playload = DevicePlayload {
                device_ip: device_info.device_ip.clone(),
                device_name: device_info.device_name.clone(),
                device_sync: device_info.device_sync.clone(),
            };

            app_handle.emit(
                "devices_event",
                serde_json::json!({
                    "event": "device_add",
                    "devices": (
                        ip.clone(), device_playload
                    )
                }),
            )?;

            map.insert(ip.clone(), device_info);
            println!("{:?}， map dev", map);
        }

        // 定时发送设备列表
        if map.len() == 1 {
            self.start_snapshot_task(app_handle.clone()).await;
        }

        Ok(())
    }
    // 移除
    pub async fn remove_device(
        &self,
        ip: &str,
        app_handle: tauri::AppHandle,
    ) -> anyhow::Result<()> {
        let mut map = self.clients.lock().await;

        if map.remove(ip).is_some() {
            app_handle.emit(
                "devices_event",
                serde_json::json!({
                    "event": "device_remove",
                    "devices": ip
                }),
            )?;
        }

        // 如果设备列表为空，停止定时任务
        if map.is_empty() {
            let mut task_guard = self.snapshot_task.lock().await;
            if let Some(handle) = task_guard.take() {
                handle.abort();
                println!("定时任务已停止");
            }
        }

        Ok(())
    }

    // 更新
    pub async fn update_device(
        &self,
        res: &DeviceInfoData,
        app_handle: tauri::AppHandle,
    ) -> anyhow::Result<()> {
        let mut map = self.clients.lock().await;
        if let Some(device) = map.get_mut(&res.device_key) {
            device.device_ip = res.device_ip.clone();
            
            // 检查是否为特殊标记，只更新同步状态
            if res.device_name != "SYNC_STATUS_UPDATE" {
                device.device_name = res.device_name.clone();
            }
            
            device.device_sync = res.file_sync;

            let payload = DevicePlayload {
                device_name: device.device_name.clone(),
                device_ip: device.device_ip.clone(),
                device_sync: device.device_sync,
            };

            let ip = res.device_key.clone();
            app_handle.emit(
                "devices_event",
                serde_json::json!({
                    "event": "device_add",
                    "devices": (
                        ip.clone(), payload
                    )
                }),
            )?;
        }

        Ok(())
    }

    pub async fn get_snapshot(&self) -> HashMap<String, DevicePlayload> {
        let map = self.clients.lock().await;
        map.iter()
            .map(|(key, val)| {
                (
                    key.clone(),
                    DevicePlayload {
                        device_ip: val.device_ip.clone(),
                        device_name: val.device_name.clone(),
                        device_sync: val.device_sync,
                    },
                )
            })
            .collect()
    }

}

impl DevicesMap {
    async fn start_snapshot_task(&self, app_handle: AppHandle) {
        let mut task_guard = self.snapshot_task.lock().await;
        if task_guard.is_none() {
            let clients = Arc::clone(&self.clients);
            let snapshot_interval_time = Arc::clone(&self.snapshot_interval_time);
            let handle = tokio::spawn(async move {
                let timer = *snapshot_interval_time.lock().await;
                let mut ticker = interval(Duration::from_secs(timer));
                loop {
                    ticker.tick().await;
                    let map = clients.lock().await;
                    if !map.is_empty() {
                        let snapshot: HashMap<String, DevicePlayload> = map
                            .iter()
                            .map(|(key, value)| {
                                (
                                    key.clone(),
                                    DevicePlayload {
                                        device_ip: value.device_ip.clone(),
                                        device_name: value.device_name.clone(),
                                        device_sync: value.device_sync.clone(),
                                    },
                                )
                            })
                            .collect();
                        if let Err(e) = app_handle.emit(
                            "devices_event",
                            serde_json::json!({
                                "event": "devices_all",
                                "devices": snapshot
                            }),
                        ) {
                            eprintln!("定时推送设备失败: {e}");
                        }
                    }
                }
            });
            *task_guard = Some(handle);
            println!("定时推送设备任务已启动");
        }
    }
}