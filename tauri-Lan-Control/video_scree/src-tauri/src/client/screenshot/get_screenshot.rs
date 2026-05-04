use screenshots::Screen;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
        Arc, Mutex,
    },
    thread,
    time::Duration,
};
use webp::Encoder;

pub struct ScreenshotStore {
    running: AtomicBool,
    paused: AtomicBool,
    handle: Mutex<Option<thread::JoinHandle<()>>>,
}

impl ScreenshotStore {
    pub fn start(screen_index: usize, interval_ms: u64, tx: Sender<Vec<u8>>) -> Arc<Self> {
        let sender = Arc::new(Self {
            running: AtomicBool::new(true),
            paused: AtomicBool::new(false),
            handle: Mutex::new(None),
        });

        // 如果interval_ms为0，表示不需要截图，直接返回暂停状态的sender
        if interval_ms == 0 {
            sender.pause();
            println!("截图器已禁用（截图间隔为0）");
            return sender;
        }

        let worker = sender.clone();

        let handle = thread::spawn(move || {
            let screens = Screen::all().expect("获取屏幕失败");
            let screen = &screens[screen_index];
            println!("启动截图器，截图间隔: {}ms", interval_ms);

            while worker.running.load(Ordering::Relaxed) {
                if worker.paused.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }

                match screen.capture() {
                    Ok(image) => {
                        let encoder =
                            Encoder::from_rgba(image.as_raw(), image.width(), image.height());

                        let bytes = encoder.encode(70.0).to_vec();

                        let _ = tx.send(bytes);
                    }
                    Err(e) => eprintln!("截图失败: {}", e),
                }

                thread::sleep(Duration::from_millis(interval_ms));
            }

            println!("截图器退出");
        });

        // 保存线程句柄
        *sender.handle.lock().unwrap() = Some(handle);
        sender
    }

    pub fn pause(&self) {
        self.paused.store(true, Ordering::Relaxed);
        println!("暂停截图器");
    }

    pub fn resume(&self) {
        self.paused.store(false, Ordering::Relaxed);
        println!("恢复截图器");
    }

    #[allow(dead_code)]
    pub fn exit(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.lock().unwrap().take() {
            let _ = handle.join();
        }
        println!("截图线程器");
    }

    // 是否暂停
    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }
}
