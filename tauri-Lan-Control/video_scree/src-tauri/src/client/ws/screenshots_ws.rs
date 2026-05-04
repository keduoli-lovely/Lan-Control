use crate::client::initialize::AppStore;
use std::{
    sync::{atomic::Ordering, mpsc, Arc, Mutex},
    thread,
    time::Duration,
};
use tauri::Emitter;
use tungstenite::{connect, Message};

pub struct ScreenshotsSender {
    tx: mpsc::Sender<Vec<u8>>,
    handle: Mutex<Option<thread::JoinHandle<()>>>,
}

fn pause_screenshot(store: &Arc<AppStore>, app_handle: tauri::AppHandle) {
    // 计数器 - 增加重试次数
    store.ws_reconnect_count.fetch_add(1, Ordering::Relaxed);
    let count = store.ws_reconnect_count.load(Ordering::Relaxed);
    let count_max = store.reconnect_count_max.load(Ordering::Relaxed);
    println!("{} - - {} --count", count, count_max);
    if count_max < count {
        // 移除所有ws
        println!("remove_ws");
        store.exit_flag.store(true, Ordering::Relaxed);

        // 发送关闭ws事件
        let _ = app_handle.emit("command", "close_ws");

        // 发送重启广播事件，让前端重新调用initialize_runtime
        let _ = app_handle.emit("command", "restart_broadcast");

        println!("已发送重启广播事件，等待重新发现服务器...");
    }
    if let Some(task) = store.screenshot_task.lock().unwrap().as_ref() {
        if !task.is_paused() {
            task.pause();
        }
    }
}

impl ScreenshotsSender {
    pub fn start(url: &str, store: Arc<AppStore>, app_handle: tauri::AppHandle) -> Arc<Self> {
        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        let url = url.to_string();

        let handle = thread::spawn(move || loop {
            // 超出次数退出循环
            if store.get_exit_flag() {
                break;
            }
            // 链接发送图片ws
            match connect(&url) {
                Ok((mut socket, _)) => {
                    println!("截图器-ws-已连接");
                    // 计数器 - 重置
                    store.ws_reconnect_count.store(0, Ordering::Relaxed);
                    // 重启截图器
                    if let Some(task) = store.screenshot_task.lock().unwrap().as_ref() {
                        if task.is_paused() {
                            task.resume();
                        }
                    }

                    loop {
                        match rx.recv_timeout(Duration::from_millis(100)) {
                            Ok(data) => {
                                if let Err(e) = socket.send(Message::Binary(data)) {
                                    eprintln!("发送失败: {}", e);
                                    break;
                                }
                            }
                            Err(mpsc::RecvTimeoutError::Timeout) => {
                                if store.get_exit_flag() {
                                    return;
                                }
                            }
                            Err(_) => break,
                        }
                    }

                    pause_screenshot(&store, app_handle.clone());
                }
                Err(e) => {
                    eprintln!("截图器-ws-连接失败: {}", e);
                    pause_screenshot(&store, app_handle.clone());

                    // 使用配置的重连间隔
                    if store.get_reconnect_ws() {
                        let reconnect_interval = store.get_reconnect_interval();
                        println!("等待 {} 秒后重试...", reconnect_interval);
                        thread::sleep(Duration::from_secs(reconnect_interval as u64));
                    } else {
                        println!("重连功能已禁用，停止重试");
                        store.exit_flag.store(true, Ordering::Relaxed);
                        break;
                    }
                }
            }
        });

        Arc::new(Self {
            tx,
            handle: Mutex::new(Some(handle)),
        })
    }

    pub fn sender(&self) -> mpsc::Sender<Vec<u8>> {
        self.tx.clone()
    }

    pub fn exit(&self) {
        if let Some(handle) = self.handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }
}
