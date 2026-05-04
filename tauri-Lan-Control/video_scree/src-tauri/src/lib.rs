// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod client;
use std::sync::Arc;

use client::{
    handle_run::{run_script, send_device_info},
    initialize::{close_all_ws, initialize_runtime, AppStore},
    utils::set_auto_run::set_auto_run,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(Arc::new(AppStore::new()))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            initialize_runtime,
            run_script,
            close_all_ws,
            send_device_info,
            set_auto_run,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
