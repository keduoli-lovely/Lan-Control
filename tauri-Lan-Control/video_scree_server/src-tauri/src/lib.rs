// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod server;

use server::{
    broadcast::broadcast_fn,
    command::{open_vnc_window, VncProxyState, test_server_status, run_system_command},
    handle_run::devices_map::DevicesMap,
    ws_server::start_background_tasks,
    create_video_data_flow, stop_fn, sned_fn, sync_devices,
    send_script_to_all, get_device_script_sync, sync_all_devices_scripts,
    trigger_file_sync, change_one_device_info,
};
use std::sync::Arc;

#[tauri::command]
async fn initialize_runtime() -> Result<(), String> {
    println!("初始化运行时...");
    // 广播服务已经在setup中启动，这里只需要返回成功
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .manage(VncProxyState::new())
        .manage(Arc::new(DevicesMap::new()))
        .invoke_handler(tauri::generate_handler![
            open_vnc_window,
            create_video_data_flow,
            stop_fn,
            sned_fn,
            sync_devices,
            test_server_status,
            run_system_command,
            send_script_to_all,
            get_device_script_sync,
            sync_all_devices_scripts,
            trigger_file_sync,
            change_one_device_info,
            initialize_runtime,
        ])
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // 启动WebSocket服务器
            start_background_tasks(app.handle().clone());
            
            // 启动UDP广播服务
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = broadcast_fn(app_handle).await {
                    eprintln!("广播服务启动失败: {}", e);
                }
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}