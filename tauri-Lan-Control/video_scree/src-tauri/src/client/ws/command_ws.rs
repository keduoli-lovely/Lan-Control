use super::super::{
    handle_run::RunCode,
    initialize::AppStore,
    utils::{
        paths::{
            ensure_resources_dir, get_client_config_path, get_client_resources_path,
            get_winvnc_path, VNC_DEFAULT_PATHS, VNC_DOWNLOAD_URL, VNC_INI_NAME, VNC_ZIP_NAME,
        },
        vnc_base64::get_vnc_data_in_base64,
        vnc_ini::get_vnc_ini,
    },
};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use reqwest::blocking::Client;
use serde_json;
use std::fs;
use std::fs::File;
use std::io::copy;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::Ordering;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message};
use zip::read::ZipArchive;

#[allow(dead_code)]
pub struct CommandClient {
    tx: mpsc::Sender<String>,
    handle: Mutex<Option<thread::JoinHandle<()>>>,
}

// 创建UltraVNC配置文件
fn create_ultravnc_config(config_path: &PathBuf, ultravnc_path: &str) -> Result<(), String> {
    let ini_content = get_vnc_ini(ultravnc_path);
    fs::write(config_path, ini_content).map_err(|e| format!("写入配置文件失败: {}", e))
}

// 获取UltraVNC路径，如果不存在则下载
fn get_or_download_vnc_path(
    app_handle: &AppHandle,
    status_tx: &mpsc::Sender<String>,
) -> Result<String, String> {
    // 首先尝试从resources目录获取
    let resources_path = get_winvnc_path(app_handle).map_err(|e| format!("路径解析失败: {}", e))?;

    if resources_path.exists() {
        println!("UltraVNC存在于resources目录: {:?}", resources_path);
        return Ok(resources_path.to_string_lossy().to_string());
    }

    // 如果不存在，尝试从已知位置获取
    for path_str in VNC_DEFAULT_PATHS {
        let path = PathBuf::from(path_str);
        if path.exists() {
            println!("UltraVNC存在于已知位置: {:?}", path);
            return Ok(path.to_string_lossy().to_string());
        }
    }

    // 如果都不存在，需要下载
    println!("UltraVNC不存在，需要下载...");

    // 发送下载开始消息
    let _ = status_tx.send("vnc_download|start".to_string());
    println!("发送下载开始消息: vnc_download|start");

    // 获取resources目录
    let resources_dir = resources_path
        .parent()
        .ok_or_else(|| "无法获取resources目录".to_string())?;

    // 创建resources目录
    if !resources_dir.exists() {
        fs::create_dir_all(resources_dir).map_err(|e| format!("创建resources目录失败: {}", e))?;
        println!("已创建resources目录: {:?}", resources_dir);
    }

    // 下载URL
    let zip_path = resources_dir.join(VNC_ZIP_NAME);

    println!("开始下载UltraVNC: {} -> {:?}", VNC_DOWNLOAD_URL, zip_path);

    // 用于保存最终下载/解码后的数据
    let result: Result<Vec<u8>, String> = (|| {
        // 尝试从网络下载
        let client = Client::new();
        let response = client
            .get(VNC_DOWNLOAD_URL)
            .send()
            .map_err(|e| format!("下载失败: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP错误: {}", response.status()));
        }

        let content = response
            .bytes()
            .map_err(|e| format!("读取响应失败: {}", e))?;
        Ok(content.to_vec())
    })()
    .or_else(|err_msg| {
        // 网络下载失败，使用 base64 回退
        let _ = status_tx.send(format!("vnc_download|fallback:{}", err_msg));
        println!("网络下载失败，尝试使用 base64 回退: {}", err_msg);

        // 获取 base64 编码的数据
        let base64_data = get_vnc_data_in_base64();

        // 解码 base64
        let decoded = STANDARD
            .decode(&base64_data)
            .map_err(|e| format!("base64解码失败: {}", e))?;
        Ok(decoded)
    });

    // 处理最终结果
    match result {
        Ok(content) => {
            // 写入文件
            if let Err(e) = std::fs::write(&zip_path, &content) {
                let err_msg = format!("写入文件失败: {}", e);
                let _ = status_tx.send(format!("vnc_download|failed:{}", err_msg));
                return Err(err_msg);
            }
            println!("文件保存成功，大小: {} 字节", content.len());
            let _ = status_tx.send("vnc_download|success".to_string());
        }
        Err(e) => {
            let err_msg = format!("获取 VNC 数据失败: {}", e);
            let _ = status_tx.send(format!("vnc_download|failed:{}", err_msg));
            return Err(err_msg);
        }
    }

    // 解压zip文件
    println!("开始解压文件...");
    let zip_file = match File::open(&zip_path) {
        Ok(f) => f,
        Err(e) => {
            let err_msg = format!("打开zip文件失败: {}", e);
            let _ = status_tx.send(format!("vnc_download|failed:{}", err_msg));
            return Err(err_msg);
        }
    };

    let mut archive = match ZipArchive::new(zip_file) {
        Ok(a) => a,
        Err(e) => {
            let err_msg = format!("读取zip档案失败: {}", e);
            let _ = status_tx.send(format!("vnc_download|failed:{}", err_msg));
            return Err(err_msg);
        }
    };

    // 解压所有文件到resources目录
    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(f) => f,
            Err(e) => {
                let err_msg = format!("读取zip条目失败: {}", e);
                let _ = status_tx.send(format!("vnc_download|failed:{}", err_msg));
                return Err(err_msg);
            }
        };
        let out_path = resources_dir.join(file.name());

        // 创建目录
        if let Some(parent) = out_path.parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    let err_msg = format!("创建目录失败: {}", e);
                    let _ = status_tx.send(format!("vnc_download|failed:{}", err_msg));
                    return Err(err_msg);
                }
            }
        }

        // 如果是目录，跳过
        if file.is_dir() {
            continue;
        }

        // 写入文件
        let mut out_file = match File::create(&out_path) {
            Ok(f) => f,
            Err(e) => {
                let err_msg = format!("创建文件失败: {}", e);
                let _ = status_tx.send(format!("vnc_download|failed:{}", err_msg));
                return Err(err_msg);
            }
        };

        if let Err(e) = copy(&mut file, &mut out_file) {
            let err_msg = format!("解压文件失败: {}", e);
            let _ = status_tx.send(format!("vnc_download|failed:{}", err_msg));
            return Err(err_msg);
        }

        println!("解压: {} -> {:?}", file.name(), out_path);
    }

    println!("解压完成");

    // 删除zip文件
    if let Err(e) = fs::remove_file(&zip_path) {
        println!("删除zip文件失败: {}, 但不影响继续执行", e);
    } else {
        println!("已删除临时zip文件");
    }

    // 再次检查winvnc.exe是否存在
    if !resources_path.exists() {
        let err_msg = "下载和解压完成后，winvnc.exe仍然不存在".to_string();
        let _ = status_tx.send(format!("vnc_download|failed:{}", err_msg));
        return Err(err_msg);
    }

    // 发送下载成功消息
    let _ = status_tx.send("vnc_download|success".to_string());
    println!("发送下载成功消息: vnc_download|success");

    println!("UltraVNC下载并解压成功: {:?}", resources_path);
    Ok(resources_path.to_string_lossy().to_string())
}

// 检查并启动UltraVNC
fn check_and_start_ultravnc(
    app_handle: &AppHandle,
    status_tx: &mpsc::Sender<String>,
) -> Result<(), String> {
    // 检查UltraVNC是否正在运行
    let output = Command::new("tasklist")
        .args(&["/fi", "IMAGENAME eq winvnc.exe"])
        .output()
        .map_err(|e| format!("检查UltraVNC进程失败: {}", e))?;

    let output_str = String::from_utf8_lossy(&output.stdout);

    if output_str.contains("winvnc.exe") {
        println!("UltraVNC已经在运行");
        return Ok(());
    }

    // 获取或下载UltraVNC路径
    let ultravnc_path = get_or_download_vnc_path(app_handle, status_tx)?;
    println!("UltraVNC路径: {}", ultravnc_path);

    // 创建配置文件
    let config_dir =
        get_client_resources_path(app_handle).map_err(|e| format!("获取配置目录失败: {}", e))?;
    let config_path = config_dir.join(VNC_INI_NAME);

    // 获取ultravnc_x64目录（winvnc.exe的父目录）
    let ultravnc_path_buf = PathBuf::from(&ultravnc_path);
    let ultravnc_dir = ultravnc_path_buf
        .parent()
        .ok_or_else(|| "无法获取UltraVNC目录".to_string())?;

    let ultravnc_dir_str = ultravnc_dir.to_string_lossy().to_string();

    create_ultravnc_config(&config_path, &ultravnc_dir_str)
        .map_err(|e| format!("创建UltraVNC配置文件失败: {}", e))?;

    println!("UltraVNC配置文件已创建: {:?}", config_path);

    // 启动UltraVNC
    println!("正在启动UltraVNC...");

    if !PathBuf::from(&ultravnc_path).exists() {
        return Err(format!("UltraVNC路径不存在: {}", ultravnc_path));
    }

    // 使用配置参数启动winvnc.exe: -run -config ini_path
    let config_path_str = config_path.to_string_lossy().to_string();
    let status = Command::new(&ultravnc_path)
        .args(&["-run", "-config", &config_path_str])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("启动UltraVNC失败: {}", e))?;

    println!("UltraVNC启动成功，进程ID: {:?}", status.id());

    // 发送VNC启动成功消息
    let _ = status_tx.send("vnc_started|success".to_string());
    println!("发送VNC启动成功消息: vnc_started|success");

    Ok(())
}

// 在同步上下文中保存脚本到Store
fn save_script_to_store(app_handle: &AppHandle, script_json: &str) -> Result<(), String> {
    println!("开始保存脚本到本地配置: {}", script_json);

    // 解析脚本JSON
    let mut script: serde_json::Value =
        serde_json::from_str(script_json).map_err(|e| format!("解析脚本JSON失败: {}", e))?;

    // 提前提取所需的值，避免借用冲突
    let script_id = script
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "脚本缺少ID字段".to_string())?
        .to_string();

    println!("脚本ID: {}", script_id);

    // 提取路径类型和原始路径
    let path_type = script
        .get("pathType")
        .and_then(|v| v.as_str())
        .unwrap_or("file")
        .to_string();

    let original_path = script
        .get("path")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    if original_path.is_empty() {
        println!("警告：脚本路径为空，ID: {}", script_id);
    } else {
        println!(
            "转换脚本路径，类型: {}, 原始路径: {}",
            path_type, original_path
        );

        // 对script进行可变操作
        if let Some(script_map) = script.as_object_mut() {
            match path_type.as_str() {
                "file" => {
                    // 文件类型：只保存文件名，不保存完整路径
                    let file_name = std::path::Path::new(&original_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");

                    script_map.insert(
                        "path".to_string(),
                        serde_json::Value::String(file_name.to_string()),
                    );
                    println!("文件类型脚本转换: {} -> {}", original_path, file_name);
                }
                "folder" => {
                    // 文件夹类型：保存文件夹名和可执行文件的相对路径
                    let folder_name = std::path::Path::new(&original_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");

                    // 保存文件夹名作为path
                    script_map.insert(
                        "path".to_string(),
                        serde_json::Value::String(folder_name.to_string()),
                    );

                    // 提取executable的值到变量，避免借用冲突
                    let executable_str = script_map
                        .get("executable")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    if !executable_str.is_empty() {
                        let folder_path = std::path::Path::new(&original_path);
                        let exec_path = std::path::Path::new(&executable_str);

                        if let Ok(relative_path) = exec_path.strip_prefix(folder_path) {
                            if let Some(relative_str) = relative_path.to_str() {
                                // 按照要求格式保存：以斜杠开头，包含文件夹名和相对路径
                                let new_executable =
                                    format!("/{}/{}", folder_name, relative_str.replace('\\', "/"));
                                script_map.insert(
                                    "executable".to_string(),
                                    serde_json::Value::String(new_executable.clone()),
                                );
                                println!(
                                    "文件夹类型脚本转换: 文件夹 {} -> {}, 可执行文件 {} -> {}",
                                    original_path, folder_name, executable_str, new_executable
                                );
                            }
                        } else {
                            // 如果无法计算相对路径，保存文件名（带文件夹前缀）
                            let exec_name = exec_path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown");
                            let new_executable = format!("/{}/{}", folder_name, exec_name);
                            script_map.insert(
                                "executable".to_string(),
                                serde_json::Value::String(new_executable.clone()),
                            );
                            println!("文件夹类型脚本转换: 文件夹 {} -> {}, 可执行文件 {} -> {} (无法计算相对路径)", 
                                original_path, folder_name, executable_str, new_executable);
                        }
                    }
                }
                _ => {
                    println!("不支持的脚本类型: {}, ID: {}", path_type, script_id);
                }
            }
        }
    }

    // 获取配置文件路径
    let config_file =
        get_client_config_path(app_handle).map_err(|e| format!("获取配置文件路径失败: {}", e))?;

    println!("配置文件路径: {:?}", config_file);

    // 创建配置目录
    if let Some(parent) = config_file.parent() {
        if !parent.exists() {
            println!("创建配置目录: {:?}", parent);
            fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {}", e))?;
            println!("配置目录创建成功");
        } else {
            println!("配置目录已存在");
        }
    }

    // 读取现有配置
    let mut config: serde_json::Map<String, serde_json::Value> = if config_file.exists() {
        println!("配置文件已存在，读取内容");
        let content =
            fs::read_to_string(&config_file).map_err(|e| format!("读取配置文件失败: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("解析配置文件失败: {}", e))?
    } else {
        println!("配置文件不存在，创建新配置");
        serde_json::Map::new()
    };

    // 获取或创建scripts对象
    let mut scripts: serde_json::Map<String, serde_json::Value> = config
        .get("scripts")
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_else(serde_json::Map::new);

    println!("当前脚本数量: {}", scripts.len());

    // 保存转换后的脚本
    scripts.insert(script_id.to_string(), script.clone());

    // 更新配置
    config.insert("scripts".to_string(), serde_json::Value::Object(scripts));

    // 保存配置到文件
    let config_content =
        serde_json::to_string_pretty(&config).map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(&config_file, config_content).map_err(|e| format!("保存配置文件失败: {}", e))?;

    println!(
        "脚本已成功保存到配置文件: ID={}, 文件={}",
        script_id,
        config_file.display()
    );
    Ok(())
}

// 获取本地所有脚本（从Store）
fn get_local_scripts(app_handle: &AppHandle) -> Result<serde_json::Value, String> {
    // 获取配置文件路径
    let config_file =
        get_client_config_path(app_handle).map_err(|e| format!("获取配置文件路径失败: {}", e))?;

    if !config_file.exists() {
        return Ok(serde_json::json!({}));
    }

    // 读取现有配置
    let content =
        fs::read_to_string(&config_file).map_err(|e| format!("读取配置文件失败: {}", e))?;
    let config: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("解析配置文件失败: {}", e))?;

    // 获取scripts对象
    let scripts = config
        .get("scripts")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));

    Ok(scripts)
}

// 检查脚本文件是否存在（包括文件夹类型）
fn check_script_files_exist(
    scripts: &serde_json::Value,
    app_handle: &AppHandle,
) -> (bool, Vec<String>) {
    let mut all_files_exist = true;
    let mut missing_files = Vec::new();

    if let Some(scripts_map) = scripts.as_object() {
        for (script_id, script) in scripts_map {
            let path_type = script
                .get("pathType")
                .and_then(|v| v.as_str())
                .unwrap_or("file");
            let original_path = script.get("path").and_then(|v| v.as_str()).unwrap_or("");

            if original_path.is_empty() {
                continue;
            }

            match path_type {
                "file" => {
                    // 检查单个文件
                    let resources_path = match get_client_resources_path(app_handle) {
                        Ok(p) => p,
                        Err(_) => {
                            all_files_exist = false;
                            missing_files.push(format!("{}: 无法获取resources目录", script_id));
                            continue;
                        }
                    };

                    let file_name = std::path::Path::new(original_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");

                    let resources_file = resources_path.join(file_name);

                    if !resources_file.exists() {
                        all_files_exist = false;
                        missing_files.push(format!("{}: {}", script_id, original_path));
                        println!("文件不存在: {:?}", resources_file);
                    } else {
                        println!("文件存在: {:?}", resources_file);
                    }
                }
                "folder" => {
                    // 检查文件夹中的可执行文件
                    let executable = script
                        .get("executable")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    if executable.is_empty() {
                        all_files_exist = false;
                        missing_files.push(format!("{}: 缺少executable字段", script_id));
                        continue;
                    }

                    let resources_path = match get_client_resources_path(app_handle) {
                        Ok(p) => p,
                        Err(_) => {
                            all_files_exist = false;
                            missing_files.push(format!("{}: 无法获取resources目录", script_id));
                            continue;
                        }
                    };

                    // 处理executable路径格式：去掉开头的斜杠
                    let exec_path_str = if executable.starts_with('/') {
                        &executable[1..]
                    } else {
                        executable
                    };

                    // 直接使用相对路径，因为executable已经是相对路径格式：ultravnc_x64/vncviewer.exe
                    let resources_exec = resources_path.join(exec_path_str);

                    if !resources_exec.exists() {
                        all_files_exist = false;
                        missing_files.push(format!("{}: {}", script_id, executable));
                        println!(
                            "可执行文件不存在: {:?} (路径: {})",
                            resources_exec, exec_path_str
                        );
                    } else {
                        println!(
                            "可执行文件存在: {:?} (路径: {})",
                            resources_exec, exec_path_str
                        );
                    }
                }
                _ => {}
            }
        }
    }

    (all_files_exist, missing_files)
}

// 同步文件到resources目录
fn sync_file_to_resources(
    relative_path: &str,
    base64_content: &str,
    app_handle: &AppHandle,
) -> Result<(), String> {
    // 确保resources目录存在
    let resources_dir = ensure_resources_dir(app_handle)?;

    // 构建目标文件路径
    let target_path = resources_dir.join(relative_path);

    // 确保目标目录存在
    if let Some(parent) = target_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| format!("创建目标目录失败: {}", e))?;
        }
    }

    // 解码base64内容
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    let file_content = STANDARD
        .decode(base64_content)
        .map_err(|e| format!("base64解码失败: {}", e))?;

    // 写入文件
    fs::write(&target_path, &file_content).map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(())
}

// 执行脚本
fn execute_script(
    script_id: &str,
    tx: &std::sync::mpsc::Sender<String>,
    app_handle: &AppHandle,
) -> Result<(), String> {
    // 获取配置文件路径
    let config_file =
        get_client_config_path(app_handle).map_err(|e| format!("获取配置文件路径失败: {}", e))?;

    if !config_file.exists() {
        // 尝试创建空的配置文件
        if let Some(parent) = config_file.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {}", e))?;
            }
        }

        // 创建空的配置文件
        let empty_config = serde_json::json!({
            "scripts": {}
        });
        let config_content = serde_json::to_string_pretty(&empty_config)
            .map_err(|e| format!("序列化空配置失败: {}", e))?;
        fs::write(&config_file, config_content).map_err(|e| format!("创建配置文件失败: {}", e))?;

        println!("已创建空的配置文件: {}", config_file.display());
        return Err(format!(
            "脚本配置为空，请先同步脚本。配置文件已创建: {}",
            config_file.display()
        ));
    }

    // 读取配置文件
    let config_content =
        fs::read_to_string(&config_file).map_err(|e| format!("读取配置文件失败: {}", e))?;

    // 解析配置JSON
    let config: serde_json::Value =
        serde_json::from_str(&config_content).map_err(|e| format!("解析配置文件失败: {}", e))?;

    // 获取scripts对象
    let scripts = config
        .get("scripts")
        .and_then(|v| v.as_object())
        .ok_or_else(|| format!("配置文件中没有scripts对象: {}", config_file.display()))?;

    // 查找指定脚本
    let script = scripts
        .get(script_id)
        .ok_or_else(|| format!("脚本ID不存在: {}", script_id))?;

    // 获取脚本信息
    let path_type = script
        .get("pathType")
        .and_then(|v| v.as_str())
        .unwrap_or("file");
    let original_path = script
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "脚本缺少path字段".to_string())?;
    let binding = vec![];
    let arguments = script
        .get("arguments")
        .and_then(|v| v.as_array())
        .unwrap_or(&binding);

    println!(
        "执行脚本: {}, 类型: {}, 原始路径: {}",
        script_id, path_type, original_path
    );

    // 检查文件是否存在，如果不存在则请求同步
    if !check_script_file_exists(script, app_handle) {
        println!("脚本文件不存在，请求同步");
        // 发送文件同步请求
        let missing_info = format!("{}:{}", script_id, original_path);
        let request_msg = format!("request_file_sync|{}", missing_info);
        let _ = tx.send(request_msg.clone());
        println!("已发送文件同步请求: {}", request_msg);
        return Err(format!("脚本文件不存在，已请求同步: {}", original_path));
    }

    // 根据路径类型执行不同的逻辑
    match path_type {
        "folder" => {
            // 文件夹模式：使用executable作为可执行文件，path作为工作目录
            let executable = script
                .get("executable")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "文件夹模式缺少executable字段".to_string())?;

            // 尝试从resources目录获取可执行文件
            let resources_path = get_client_resources_path(app_handle)?;

            // 处理executable路径格式：去掉开头的斜杠
            let exec_path_str = if executable.starts_with('/') {
                &executable[1..]
            } else {
                executable
            };

            // 直接使用相对路径，因为executable已经是相对路径格式：ultravnc_x64/vncviewer.exe
            let resources_executable = resources_path.join(exec_path_str);

            // 工作目录：使用原始路径（文件夹路径）
            let work_dir = PathBuf::from(original_path);

            let exec_path = if resources_executable.exists() {
                println!(
                    "使用resources目录中的可执行文件: {:?}",
                    resources_executable
                );
                resources_executable
            } else {
                println!("resources目录中不存在，使用原始路径: {}", executable);
                PathBuf::from(executable)
            };

            // 确保工作目录存在
            if !work_dir.exists() {
                println!("警告：工作目录不存在: {:?}", work_dir);
                // 尝试创建工作目录
                if let Err(e) = fs::create_dir_all(&work_dir) {
                    println!("创建工作目录失败: {}", e);
                }
            }

            println!(
                "执行文件夹脚本: 工作目录={:?}, 可执行文件={:?}",
                work_dir, exec_path
            );

            let mut cmd = Command::new(exec_path);
            cmd.current_dir(work_dir);

            // 添加参数
            for arg in arguments {
                if let Some(arg_str) = arg.as_str() {
                    cmd.arg(arg_str);
                }
            }

            // 执行命令
            match cmd.spawn() {
                Ok(mut child) => {
                    println!("脚本执行成功，进程ID: {:?}", child.id());
                    // 不等待进程结束，让它后台运行
                    std::thread::spawn(move || {
                        let _ = child.wait();
                    });
                    Ok(())
                }
                Err(e) => {
                    let err_msg = format!("执行脚本失败: {}", e);
                    println!("{}", err_msg);
                    Err(err_msg)
                }
            }
        }
        "file" => {
            // 文件模式：直接执行文件
            // 尝试从resources目录获取文件
            let resources_path = get_client_resources_path(app_handle)?;

            // 获取文件名
            let original_path_buf = PathBuf::from(original_path);
            let file_name = original_path_buf
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            let resources_file = resources_path.join(file_name);

            // 优先使用resources目录中的文件，如果不存在则使用原始路径
            let exec_path = if resources_file.exists() {
                println!("使用resources目录中的文件: {:?}", resources_file);
                resources_file
            } else {
                println!("resources目录中不存在，使用原始路径: {}", original_path);
                PathBuf::from(original_path)
            };

            println!("执行文件脚本: 路径={:?}", exec_path);

            let mut cmd = Command::new(exec_path);

            // 添加参数
            for arg in arguments {
                if let Some(arg_str) = arg.as_str() {
                    cmd.arg(arg_str);
                }
            }

            // 执行命令
            match cmd.spawn() {
                Ok(mut child) => {
                    println!("脚本执行成功，进程ID: {:?}", child.id());
                    // 不等待进程结束，让它后台运行
                    std::thread::spawn(move || {
                        let _ = child.wait();
                    });
                    Ok(())
                }
                Err(e) => Err(format!("执行脚本失败: {}", e)),
            }
        }
        _ => Err(format!("不支持的路径类型: {}", path_type)),
    }
}

// 检查单个脚本文件是否存在
fn check_script_file_exists(script: &serde_json::Value, app_handle: &AppHandle) -> bool {
    let path_type = script
        .get("pathType")
        .and_then(|v| v.as_str())
        .unwrap_or("file");
    let original_path = script.get("path").and_then(|v| v.as_str()).unwrap_or("");

    if original_path.is_empty() {
        return false;
    }

    match path_type {
        "file" => {
            // 检查单个文件
            let resources_path = match get_client_resources_path(app_handle) {
                Ok(p) => p,
                Err(_) => return false,
            };

            let file_name = std::path::Path::new(original_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            let resources_file = resources_path.join(file_name);

            if !resources_file.exists() {
                println!("文件不存在: {:?}", resources_file);
                return false;
            }
            true
        }
        "folder" => {
            // 检查文件夹中的可执行文件
            let executable = script
                .get("executable")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if executable.is_empty() {
                return false;
            }

            let resources_path = match get_client_resources_path(app_handle) {
                Ok(p) => p,
                Err(_) => return false,
            };

            // 处理executable路径格式：去掉开头的斜杠
            let exec_path_str = if executable.starts_with('/') {
                &executable[1..]
            } else {
                executable
            };

            // 直接使用相对路径，因为executable已经是相对路径格式：ultravnc_x64/vncviewer.exe
            let resources_exec = resources_path.join(exec_path_str);

            if !resources_exec.exists() {
                println!(
                    "可执行文件不存在: {:?} (路径: {})",
                    resources_exec, exec_path_str
                );
                return false;
            }
            true
        }
        _ => false,
    }
}

// 发送脚本同步状态到服务器（基于实际文件检查）
fn send_script_sync_status(
    tx: &mpsc::Sender<String>,
    scripts: &serde_json::Value,
    app_handle: &AppHandle,
) {
    // 防抖机制：使用线程本地存储记录上次发送状态和时间
    use std::cell::RefCell;
    thread_local! {
        static LAST_SYNC_STATUS: RefCell<Option<bool>> = RefCell::new(None);
        static LAST_SYNC_TIME: RefCell<std::time::Instant> = RefCell::new(std::time::Instant::now());
    }
    const DEBOUNCE_INTERVAL_MS: u64 = 2000; // 2秒防抖间隔

    // 内部发送函数，带防抖检查
    let send_status = |status: bool| {
        let now = std::time::Instant::now();
        let mut should_send = true;

        LAST_SYNC_STATUS.with(|last_status_cell| {
            LAST_SYNC_TIME.with(|last_time_cell| {
                let last_status = *last_status_cell.borrow();
                let last_time = *last_time_cell.borrow();

                should_send = match last_status {
                    Some(last_status_val) => {
                        // 如果状态改变，或者超过防抖间隔时间
                        last_status_val != status
                            || now.duration_since(last_time).as_millis()
                                >= DEBOUNCE_INTERVAL_MS as u128
                    }
                    None => true, // 第一次发送
                };

                if should_send {
                    *last_status_cell.borrow_mut() = Some(status);
                    *last_time_cell.borrow_mut() = now;
                }
            });
        });

        if should_send {
            let message = if status {
                "device_sync|true"
            } else {
                "device_sync|false"
            };
            let _ = tx.send(message.to_string());
            println!("发送同步状态: {}", message);
            true
        } else {
            println!(
                "跳过发送重复的同步状态: {}",
                if status {
                    "device_sync|true"
                } else {
                    "device_sync|false"
                }
            );
            false
        }
    };

    // 检查脚本配置是否存在
    let has_scripts = !scripts.as_object().unwrap().is_empty();

    if !has_scripts {
        // 没有脚本配置：发送未同步状态，并主动请求脚本
        send_status(false);
        // 主动发送get_script请求，确保服务器发送脚本配置
        let _ = tx.send("get_script".to_string());
        println!("发送脚本同步状态: device_sync|false (无脚本配置，已发送get_script请求)");
        return;
    }

    // 检查resources目录是否存在
    match get_client_resources_path(app_handle) {
        Ok(resources_path) => {
            if !resources_path.exists() {
                // resources目录不存在，发送未同步状态并请求同步
                println!("resources目录不存在: {:?}", resources_path);
                send_status(false);
                // 请求所有脚本的文件同步
                let mut missing_list = String::new();
                if let Some(scripts_map) = scripts.as_object() {
                    let script_ids: Vec<String> = scripts_map.keys().cloned().collect();
                    missing_list = script_ids.join(";");
                }
                if !missing_list.is_empty() {
                    let request_msg = format!("request_file_sync|{}", missing_list);
                    let _ = tx.send(request_msg.clone());
                    println!("发送文件同步请求(resources目录不存在): {}", request_msg);
                }
                println!("发送脚本同步状态: device_sync|false (resources目录不存在)");
                return;
            }
        }
        Err(e) => {
            println!("获取resources目录路径失败: {}, 发送未同步状态", e);
            send_status(false);
            return;
        }
    }

    // 检查实际文件是否存在
    let (all_files_exist, missing_files) = check_script_files_exist(scripts, app_handle);

    if !all_files_exist {
        println!("文件缺失: {:?}", missing_files);
        // 发送文件缺失状态，同时请求同步
        let missing_list = missing_files.join(";");
        let request_msg = format!("request_file_sync|{}", missing_list);
        let _ = tx.send(request_msg.clone());
        println!("发送文件同步请求: {}", request_msg);

        // 同时也发送未同步状态
        send_status(false);
        println!("发送脚本同步状态: device_sync|false (文件缺失)");
    } else {
        // 所有文件都存在，发送同步成功状态
        send_status(true);
        println!("发送脚本同步状态: device_sync|true (所有文件存在)");
    }
}

impl CommandClient {
    pub fn start(url: &str, store: Arc<AppStore>, app_handle: AppHandle) -> Arc<Self> {
        let (tx, rx) = mpsc::channel::<String>();
        let url = url.to_string();

        // 保存一份tx的克隆用于结构体，因为闭包会移动tx
        let struct_tx = tx.clone();

        let handle = thread::spawn(move || {
            // 创建客户端内部的tx用于发送状态消息
            let status_tx = tx.clone();

            // 发送初始同步状态 - 默认发送未同步状态，强制触发文件同步检查
            // 无论是否有脚本配置，都发送false，这样服务器会发送脚本和触发文件同步
            let _ = status_tx.send("device_sync|false".to_string());
            println!("发送初始同步状态: device_sync|false (强制触发文件同步)");

            // 同时发送get_script请求，确保服务器发送脚本配置
            let _ = status_tx.send("get_script".to_string());
            println!("已发送get_script请求");

            loop {
                // 超出次数退出循环
                if store.get_exit_flag() {
                    break;
                }
                // 链接发送指令 ws
                match connect(&url) {
                    Ok((mut socket, _)) => {
                        println!("command-ws-已连接");

                        if let MaybeTlsStream::Plain(stream) = socket.get_mut() {
                            stream.set_nonblocking(true).unwrap();
                        }

                        // 连接建立后立即发送一次同步状态检查
                        match get_local_scripts(&app_handle) {
                            Ok(scripts) => {
                                let has_scripts = !scripts.as_object().unwrap().is_empty();
                                send_script_sync_status(&status_tx, &scripts, &app_handle);
                                println!("连接建立后发送初始同步状态检查");

                                // 如果没有脚本，主动请求服务器发送脚本
                                if !has_scripts {
                                    println!("本地没有脚本，主动请求服务器发送脚本");
                                    let _ = status_tx.send("get_script".to_string());
                                    println!("已发送get_script请求");
                                }
                            }
                            Err(e) => {
                                eprintln!("检查本地脚本失败: {}", e);
                                // 创建空的scripts对象用于错误情况
                                let empty_scripts = serde_json::json!({});
                                send_script_sync_status(&status_tx, &empty_scripts, &app_handle);

                                // 即使检查失败也主动请求脚本
                                println!("本地脚本检查失败，主动请求服务器发送脚本");
                                let _ = status_tx.send("get_script".to_string());
                                println!("已发送get_script请求");
                            }
                        }

                        loop {
                            let mut active = false;

                            // 接收服务器消息
                            match socket.read() {
                                Ok(msg) => {
                                    active = true;
                                    match msg {
                                        Message::Text(text) => {
                                            // 检查消息类型，避免打印包含base64的完整消息
                                            if text.starts_with("file_sync|") {
                                                // 对于文件同步消息，只打印路径信息，不打印base64内容
                                                let parts: Vec<&str> =
                                                    text.splitn(2, '|').collect();
                                                if parts.len() == 2 {
                                                    let file_info = parts[0]; // 只获取第一部分
                                                    println!(
                                                        "收到文件同步命令，文件: {}",
                                                        file_info
                                                    );
                                                } else {
                                                    println!("收到文件同步命令");
                                                }
                                            } else {
                                                println!("收到文本消息: {}", text);
                                            }

                                            // 检查是否为脚本同步命令
                                            if text.starts_with("script_sync|") {
                                                let script_json =
                                                    text.trim_start_matches("script_sync|");
                                                println!("收到脚本同步命令: {}", script_json);

                                                // 保存脚本配置
                                                match save_script_to_store(&app_handle, script_json)
                                                {
                                                    Ok(_) => {
                                                        println!("脚本配置保存成功");
                                                        // 保存成功后，检查文件是否存在并发送正确的状态
                                                        match get_local_scripts(&app_handle) {
                                                            Ok(scripts) => {
                                                                send_script_sync_status(
                                                                    &status_tx,
                                                                    &scripts,
                                                                    &app_handle,
                                                                );
                                                                println!("脚本配置保存后，已检查文件存在性并发送状态");
                                                            }
                                                            Err(e) => {
                                                                eprintln!(
                                                                    "保存后获取本地脚本失败: {}",
                                                                    e
                                                                );
                                                                let _ = status_tx.send(
                                                                    "device_sync|false".to_string(),
                                                                );
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        eprintln!("脚本配置保存失败: {}", e);
                                                        let _ = status_tx
                                                            .send("device_sync|false".to_string());
                                                    }
                                                }
                                            } else if text.starts_with("file_sync|") {
                                                // 文件同步命令
                                                let file_data =
                                                    text.trim_start_matches("file_sync|");

                                                // 解析文件数据：格式为 "relative_path|base64_content"
                                                let parts: Vec<&str> =
                                                    file_data.splitn(2, '|').collect();
                                                if parts.len() == 2 {
                                                    let relative_path = parts[0];
                                                    // 只打印路径，不打印base64内容
                                                    println!("收到文件同步命令: {}", relative_path);

                                                    let base64_content = parts[1];

                                                    match sync_file_to_resources(
                                                        relative_path,
                                                        base64_content,
                                                        &app_handle,
                                                    ) {
                                                        Ok(_) => {
                                                            println!(
                                                                "文件同步成功: {}",
                                                                relative_path
                                                            );

                                                            // 文件同步完成后，检查所有脚本文件是否已同步完成
                                                            match get_local_scripts(&app_handle) {
                                                                Ok(scripts) => {
                                                                    let (all_files_exist, _) =
                                                                        check_script_files_exist(
                                                                            &scripts,
                                                                            &app_handle,
                                                                        );
                                                                    if all_files_exist {
                                                                        // 所有文件都已同步，使用防抖机制发送同步完成状态
                                                                        send_script_sync_status(
                                                                            &status_tx,
                                                                            &scripts,
                                                                            &app_handle,
                                                                        );
                                                                        println!("所有文件同步完成，已发送同步状态");
                                                                    }
                                                                }
                                                                Err(e) => {
                                                                    eprintln!(
                                                                        "检查本地脚本失败: {}",
                                                                        e
                                                                    );
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            eprintln!("文件同步失败: {}", e);
                                                        }
                                                    }
                                                } else {
                                                    eprintln!(
                                                        "文件同步命令格式错误: {}",
                                                        file_data
                                                    );
                                                }
                                            } else if text == "script_check_sync" {
                                                println!("收到脚本检查同步命令，重新检查所有文件");
                                                // 检查本地脚本状态并发送
                                                match get_local_scripts(&app_handle) {
                                                    Ok(scripts) => {
                                                        // 强制重新检查所有文件，而不仅仅是配置文件
                                                        let (all_files_exist, _) =
                                                            check_script_files_exist(
                                                                &scripts,
                                                                &app_handle,
                                                            );
                                                        println!(
                                                            "重新检查文件结果: 所有文件存在? {}",
                                                            all_files_exist
                                                        );

                                                        if all_files_exist {
                                                            // 所有文件都存在，使用防抖机制发送同步成功状态
                                                            send_script_sync_status(
                                                                &status_tx,
                                                                &scripts,
                                                                &app_handle,
                                                            );
                                                            println!(
                                                                "文件已全部同步，已发送同步状态"
                                                            );
                                                        } else {
                                                            // 文件缺失，发送未同步状态
                                                            send_script_sync_status(
                                                                &status_tx,
                                                                &scripts,
                                                                &app_handle,
                                                            );
                                                        }
                                                    }
                                                    Err(e) => {
                                                        eprintln!("检查本地脚本失败: {}", e);
                                                        // 创建空的scripts对象用于错误情况
                                                        let empty_scripts = serde_json::json!({});
                                                        send_script_sync_status(
                                                            &status_tx,
                                                            &empty_scripts,
                                                            &app_handle,
                                                        );
                                                    }
                                                }
                                            } else if text.starts_with("get_script") {
                                                // 获取脚本请求
                                                println!("收到获取脚本命令");
                                                match get_local_scripts(&app_handle) {
                                                    Ok(scripts) => {
                                                        // 发送所有脚本到服务器（控制端），格式为 "script|{json}"
                                                        let scripts_str =
                                                            serde_json::to_string(&scripts)
                                                                .unwrap_or_default();
                                                        let message =
                                                            format!("script|{}", scripts_str);
                                                        let _ = status_tx.send(message.clone());
                                                        println!("已发送脚本数据到服务器（控制端），格式: script|{}", scripts_str);
                                                    }
                                                    Err(e) => {
                                                        eprintln!("获取本地脚本失败: {}", e);
                                                    }
                                                }
                                            } else if text.starts_with("script_execute|") {
                                                // 执行脚本命令
                                                let script_id =
                                                    text.trim_start_matches("script_execute|");
                                                println!("收到执行脚本命令: {}", script_id);

                                                match execute_script(
                                                    script_id,
                                                    &status_tx,
                                                    &app_handle,
                                                ) {
                                                    Ok(_) => {
                                                        println!("脚本执行成功: {}", script_id);
                                                        // 可以发送执行成功状态给服务器
                                                        let _ = status_tx.send(format!(
                                                            "script_executed|{}|success",
                                                            script_id
                                                        ));
                                                    }
                                                    Err(e) => {
                                                        eprintln!("脚本执行失败: {}", e);
                                                        let _ = status_tx.send(format!(
                                                            "script_executed|{}|failed:{}",
                                                            script_id, e
                                                        ));
                                                    }
                                                }
                                            } else {
                                                // 处理其他命令
                                                match text.as_str() {
                                                    "shutdown" => {
                                                        RunCode::shutdown().runing().unwrap();
                                                    }
                                                    "reboot" => {
                                                        RunCode::reboot().runing().unwrap();
                                                    }
                                                    "see" => {
                                                        // 检查pause_preview设置，如果需要则暂停截图
                                                        {
                                                            let server_info =
                                                                store.server_info.lock().unwrap();
                                                            if server_info.pause_preview {
                                                                println!("远程时暂停预览功能已启用，暂停截图");

                                                                // 暂停截图任务
                                                                if let Some(task) = store
                                                                    .screenshot_task
                                                                    .lock()
                                                                    .unwrap()
                                                                    .as_ref()
                                                                {
                                                                    if !task.is_paused() {
                                                                        task.pause();
                                                                        println!("截图器已暂停");
                                                                        // VNC窗口关闭时发送resume_screenshot命令
                                                                        println!("等待VNC窗口关闭事件恢复截图");
                                                                    } else {
                                                                        println!(
                                                                            "截图器已处于暂停状态"
                                                                        );
                                                                    }
                                                                } else {
                                                                    println!(
                                                                        "截图器未初始化，无法暂停"
                                                                    );
                                                                }
                                                            } else {
                                                                println!("远程时暂停预览功能未启用，继续截图");
                                                            }
                                                        }

                                                        // 检查并启动UltraVNC
                                                        match check_and_start_ultravnc(
                                                            &app_handle,
                                                            &status_tx,
                                                        ) {
                                                            Ok(_) => println!("UltraVNC已启动"),
                                                            Err(e) => {
                                                                eprintln!("启动UltraVNC失败: {}", e)
                                                            }
                                                        }
                                                    }
                                                    "resume_screenshot" => {
                                                        println!("收到恢复截图命令");
                                                        if let Some(task) = store
                                                            .screenshot_task
                                                            .lock()
                                                            .unwrap()
                                                            .as_ref()
                                                        {
                                                            if task.is_paused() {
                                                                task.resume();
                                                                println!("截图器已恢复");
                                                            } else {
                                                                println!("截图器未处于暂停状态");
                                                            }
                                                        } else {
                                                            println!("截图器未初始化");
                                                        }
                                                    }
                                                    "sync_files" => {
                                                        println!(
                                                            "收到同步文件指令，开始执行文件同步"
                                                        );

                                                        // 检查本地脚本并执行实际的文件同步检查
                                                        match get_local_scripts(&app_handle) {
                                                            Ok(scripts) => {
                                                                let has_scripts = !scripts
                                                                    .as_object()
                                                                    .unwrap()
                                                                    .is_empty();

                                                                if has_scripts {
                                                                    // 有脚本配置，检查实际文件并发送请求
                                                                    send_script_sync_status(
                                                                        &status_tx,
                                                                        &scripts,
                                                                        &app_handle,
                                                                    );
                                                                    println!("已执行文件同步检查并发送状态");
                                                                } else {
                                                                    // 没有脚本配置，请求获取脚本
                                                                    let _ = status_tx.send(
                                                                        "get_script".to_string(),
                                                                    );
                                                                    println!("没有本地脚本，已发送get_script请求");
                                                                }
                                                            }
                                                            Err(e) => {
                                                                eprintln!(
                                                                    "检查本地脚本失败: {}",
                                                                    e
                                                                );
                                                                // 即使检查失败，也请求脚本
                                                                let _ = status_tx
                                                                    .send("get_script".to_string());
                                                                println!("脚本检查失败，已发送get_script请求");
                                                            }
                                                        }
                                                    }
                                                    other => {
                                                        let _ = app_handle.emit("command", other);
                                                    }
                                                }
                                            }
                                        }
                                        Message::Close(_) => {
                                            break;
                                        }
                                        _ => {}
                                    }
                                }
                                Err(tungstenite::Error::Io(ref e))
                                    if e.kind() == std::io::ErrorKind::WouldBlock => {}
                                Err(e) => {
                                    eprintln!("command-读取消息出错: {}", e);
                                    break;
                                }
                            }

                            // 发送待发的指令消息
                            while let Ok(cmd) = rx.try_recv() {
                                active = true;
                                if let Err(e) = socket.send(Message::Text(cmd)) {
                                    eprintln!("command-发送失败: {}", e);
                                    break;
                                }
                            }

                            if !active {
                                thread::sleep(Duration::from_millis(1));
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("command-ws-连接失败: {}", e);

                        // 使用配置的重连间隔
                        if store.get_reconnect_ws() {
                            let reconnect_interval = store.get_reconnect_interval();
                            println!("CommandClient等待 {} 秒后重试...", reconnect_interval);
                            thread::sleep(Duration::from_secs(reconnect_interval as u64));
                        } else {
                            println!("CommandClient重连功能已禁用，停止重试");
                            store.exit_flag.store(true, Ordering::Relaxed);
                            break;
                        }
                    }
                }
            }
        });

        Arc::new(Self {
            tx: struct_tx,
            handle: Mutex::new(Some(handle)),
        })
    }

    pub fn send_command(&self, cmd: String) {
        let _ = self.tx.send(cmd);
    }

    pub fn exit(&self) {
        if let Some(handle) = self.handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }
}
