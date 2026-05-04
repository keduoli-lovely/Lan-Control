use std::path::{Path, PathBuf};
use std::fs;
use serde_json;
use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD};

pub fn get_client_resources_path() -> Result<PathBuf, String> {
    let app_data = std::env::var("APPDATA")
        .map_err(|e| format!("获取APPDATA环境变量失败: {}", e))?;
    let resources_dir = PathBuf::from(&app_data).join("com.keduoli.video_scree").join("resources");
    Ok(resources_dir)
}

pub fn ensure_resources_dir() -> Result<PathBuf, String> {
    let resources_path = get_client_resources_path()?;
    if !resources_path.exists() {
        fs::create_dir_all(&resources_path)
            .map_err(|e| format!("创建resources目录失败: {}", e))?;
    }
    Ok(resources_path)
}

// 计算文件的MD5哈希（用于检测变化）
pub fn compute_file_hash(file_path: &Path) -> Result<String, String> {
    use std::io::Read;
    
    if !file_path.exists() {
        return Err("文件不存在".to_string());
    }
    
    let mut file = fs::File::open(file_path)
        .map_err(|e| format!("打开文件失败: {}", e))?;
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| format!("读取文件失败: {}", e))?;
    
    let hash = md5::compute(&buffer);
    Ok(format!("{:x}", hash))
}

// 同步文件到被控端resources目录
pub fn sync_file_to_resources(source_path: &Path, relative_path: &str) -> Result<(), String> {
    // 获取目标路径
    let resources_dir = ensure_resources_dir()?;
    let target_path = resources_dir.join(relative_path);
    
    // 确保目标目录存在
    if let Some(parent) = target_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("创建目标目录失败: {}", e))?;
        }
    }
    
    // 如果源文件不存在，则删除目标文件
    if !source_path.exists() {
        if target_path.exists() {
            fs::remove_file(&target_path)
                .map_err(|e| format!("删除目标文件失败: {}", e))?;
            println!("删除文件: {:?} -> {:?}", source_path, target_path);
        }
        return Ok(());
    }
    
    // 检查文件是否已经存在且相同
    if target_path.exists() {
        let source_hash = compute_file_hash(source_path);
        let target_hash = compute_file_hash(&target_path);
        
        if let (Ok(source_hash), Ok(target_hash)) = (&source_hash, &target_hash) {
            if source_hash == target_hash {
                // 文件相同，无需同步
                return Ok(());
            }
        }
    }
    
    // 复制文件
    fs::copy(source_path, &target_path)
        .map_err(|e| format!("复制文件失败: {}", e))?;
    
    println!("同步文件: {:?} -> {:?}", source_path, target_path);
    Ok(())
}

// 读取文件并转换为base64字符串
pub fn read_file_as_base64(file_path: &Path) -> Result<String, String> {
    use std::io::Read;
    
    if !file_path.exists() {
        return Err(format!("文件不存在: {}", file_path.display()));
    }
    
    let mut file = fs::File::open(file_path)
        .map_err(|e| format!("打开文件失败: {}", e))?;
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| format!("读取文件失败: {}", e))?;
    
    // 将文件内容编码为base64
    let base64_content = STANDARD.encode(&buffer);
    Ok(base64_content)
}
// 同步脚本相关的文件（如果脚本中有文件路径）
pub fn sync_script_files(script: &serde_json::Value) -> Result<(), String> {
    // 获取脚本中的文件路径
    let path = script.get("path")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    if path.is_empty() {
        return Ok(());
    }
    
    let path_type = script.get("pathType")
        .and_then(|v| v.as_str())
        .unwrap_or("file");
    
    match path_type {
        "file" => {
            // 同步单个文件
            let source_path = PathBuf::from(path);
            if source_path.exists() {
                let file_name = source_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                
                sync_file_to_resources(&source_path, file_name)?;
            }
        }
        "folder" => {
            // 同步文件夹中的可执行文件
            let executable = script.get("executable")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            if !executable.is_empty() {
                let source_path = PathBuf::from(executable);
                if source_path.exists() {
                    // 获取相对于脚本路径的相对路径
                    let folder_path = PathBuf::from(path);
                    let exec_path = PathBuf::from(executable);
                    
                    if let Ok(relative_path) = exec_path.strip_prefix(&folder_path) {
                        if let Some(relative_str) = relative_path.to_str() {
                            sync_file_to_resources(&exec_path, relative_str)?;
                        }
                    }
                }
            }
        }
        _ => {}
    }
    
    Ok(())
}
