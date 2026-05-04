use std::fs;
use std::path::PathBuf;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

// vnc 下载网址
pub const VNC_DOWNLOAD_URL: &str = "http://keduoli.com/d/A/UltraVNC/ultravnc_x64.zip";
// vnc 默认路径查找
pub const VNC_DEFAULT_PATHS: [&str; 2] = [
    r"C:\Program Files\UltraVNC\winvnc.exe",
    r"C:\Program Files (x86)\UltraVNC\winvnc.exe",
];
// 下载 vnc zip 昵称
pub const VNC_ZIP_NAME: &str = "ultravnc_x64.zip";
// vnc ini 配置文件名
pub const VNC_INI_NAME: &str = "ultravnc.ini";
// 客户端配置文件名
const CLIENT_SETTINGS_FILE: &str = ".Client_settings.json";

// 获取vnc winvnc.exe路径
pub fn get_winvnc_path(app_handle: &AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(app_handle
        .path()
        .resolve("resources/ultravnc_x64/winvnc.exe", BaseDirectory::Resource)?)
}

// 获取客户端resources目录路径
pub fn get_client_resources_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    // 将文件保存到resources目录
    let resources_dir = app_handle
        .path()
        .resolve("resources", BaseDirectory::Resource)
        .map_err(|e| format!("解析resources目录失败: {}", e))?;

    // 创建resources目录（如果不存在）
    if !resources_dir.exists() {
        fs::create_dir_all(&resources_dir).map_err(|e| format!("创建resources目录失败: {}", e))?;
    }

    Ok(resources_dir)
}

// 校验resources目录存在
pub fn ensure_resources_dir(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let resources_path = get_client_resources_path(app_handle)?;
    if !resources_path.exists() {
        println!("创建resources目录: {:?}", resources_path);
        fs::create_dir_all(&resources_path).map_err(|e| format!("创建resources目录失败: {}", e))?;
        println!("resources目录创建成功");
    }
    Ok(resources_path)
}

// 获取resources路径拼接配置文件路径
pub fn get_client_config_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    // 将配置文件保存到resources目录
    let resources_dir = get_client_resources_path(app_handle)
        .map_err(|e| format!("解析resources目录失败: {}", e))?;

    // 创建resources目录（如果不存在）
    if !resources_dir.exists() {
        fs::create_dir_all(&resources_dir).map_err(|e| format!("创建resources目录失败: {}", e))?;
    }

    Ok(resources_dir.join(CLIENT_SETTINGS_FILE))
}
