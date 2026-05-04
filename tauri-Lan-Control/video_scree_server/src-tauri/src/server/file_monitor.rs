use std::path::PathBuf;
use serde_json;
use std::fs;
use anyhow::Result;

pub fn get_server_config_path() -> Result<PathBuf, String> {
    let app_data = std::env::var("APPDATA")
        .map_err(|e| format!("获取APPDATA环境变量失败: {}", e))?;
    let config_dir = PathBuf::from(&app_data).join("com.keduoli.video_scree_server");
    Ok(config_dir.join(".server_settings.json"))
}

pub fn ensure_config_dir() -> Result<PathBuf, String> {
    let config_path = get_server_config_path()?;
    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("创建配置目录失败: {}", e))?;
        }
    }
    Ok(config_path)
}

pub fn read_config_file() -> Result<serde_json::Value, String> {
    let config_path = get_server_config_path()?;
    if !config_path.exists() {
        return Ok(serde_json::json!({}));
    }
    
    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;
    
    serde_json::from_str(&content)
        .map_err(|e| format!("解析配置文件失败: {}", e))
}

pub fn save_config_file(config: &serde_json::Value) -> Result<(), String> {
    let config_path = ensure_config_dir()?;
    
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    
    fs::write(&config_path, content)
        .map_err(|e| format!("保存配置文件失败: {}", e))
}

pub fn get_scripts_config() -> Result<serde_json::Value, String> {
    let config = read_config_file()?;
    
    Ok(config.get("scripts")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({})))
}

pub fn save_script(script_id: &str, script: &serde_json::Value) -> Result<(), String> {
    let mut config = read_config_file()?;
    
    // 获取或创建scripts对象
    let mut scripts: serde_json::Map<String, serde_json::Value> = config
        .get("scripts")
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_else(serde_json::Map::new);
    
    // 保存脚本
    scripts.insert(script_id.to_string(), script.clone());
    
    // 更新配置
    config.as_object_mut()
        .ok_or_else(|| "配置不是对象".to_string())?
        .insert("scripts".to_string(), serde_json::Value::Object(scripts));
    
    save_config_file(&config)
}
