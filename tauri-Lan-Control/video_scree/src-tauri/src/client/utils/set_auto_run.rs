use crate::client::initialize::AppStore;
use std::sync::Arc;
use tauri_plugin_autostart::ManagerExt;

// autostart 设置开机自启
#[tauri::command]
pub fn set_auto_run(
    app_handle: tauri::AppHandle,
    store: tauri::State<'_, Arc<AppStore>>,
    state: bool,
) -> Result<(), String> {
    // 根据store中的auto_start设置是否开机自启
    let autostart_manager = app_handle.autolaunch();
    match autostart_manager.is_enabled() {
        Ok(true) => {
            println!("关闭开机自启");
            if !state {
                let _ = autostart_manager.disable();
            }
        }
        Ok(false) => {
            println!("开启开机自启");
            if state {
                let _ = autostart_manager.enable();
            }
        }
        Err(e) => {
            println!("获取开机自启状态失败: {}", e);
        }
    }

    // 更新store中的auto_start状态
    if let Ok(mut store) = store.server_info.lock() {
        store.auto_start = state;
    }

    Ok(())
}
