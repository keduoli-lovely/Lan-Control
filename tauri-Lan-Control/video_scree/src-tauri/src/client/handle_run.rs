use super::utils::paths::get_client_config_path;
use std::process::Command;

#[tauri::command]
pub async fn send_device_info(
    device_key: String,
    device_name: String,
    store: tauri::State<'_, std::sync::Arc<crate::client::initialize::AppStore>>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // 获取当前设备的本地IP地址
    let device_ip = match get_local_ip() {
        Some(ip) => ip,
        None => {
            eprintln!("无法获取本地IP地址");
            "127.0.0.1".to_string()
        }
    };

    // 检查本地是否有脚本
    let has_scripts = get_client_config_path(&app_handle)
        .map_err(|e| format!("resources目录获取失败: {}", e))?
        .exists();
    println!(
        "发送设备信息: key={}, name={}, ip={}, has_scripts={:?}",
        device_key, device_name, device_ip, has_scripts
    );

    // 检查是否有连接的command_client
    if let Some(command_client) = store.command_client.lock().unwrap().as_ref() {
        // 构建设备信息JSON，根据本地脚本状态设置file_sync
        let device_info = serde_json::json!({
            "device_key": device_key,
            "device_name": device_name,
            "device_ip": device_ip,
            "file_sync": has_scripts
        });

        // 发送设备信息到服务器
        let message = format!("device_info|{}", device_info.to_string());
        command_client.send_command(message);

        Ok(())
    } else {
        Err("没有连接到服务器的CommandClient".to_string())
    }
}

// 获取本地IP地址
fn get_local_ip() -> Option<String> {
    use std::net::UdpSocket;
    use std::time::Duration;

    // 尝试获取本地IP地址
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        socket.set_read_timeout(Some(Duration::from_secs(1))).ok()?;

        // 尝试连接到一个已知地址（Google DNS）
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                return Some(addr.ip().to_string());
            }
        }
    }

    // 获取网络接口IP
    if let Ok(ifaces) = get_if_addrs::get_if_addrs() {
        for iface in ifaces {
            if !iface.is_loopback() && iface.ip().is_ipv4() {
                return Some(iface.ip().to_string());
            }
        }
    }

    None
}
pub struct RunCode {
    file_path: String,
    code: Vec<String>,
}

impl RunCode {
    pub fn new(file_path: String, code: Vec<String>) -> Self {
        Self { file_path, code }
    }

    pub fn shutdown() -> Self {
        Self {
            file_path: "cmd.exe".to_string(),
            code: vec![
                "/C".to_string(),
                "shutdown".to_string(),
                "-s".to_string(),
                "-t".to_string(),
                "0".to_string(),
            ],
        }
    }

    pub fn reboot() -> Self {
        Self {
            file_path: "cmd.exe".to_string(),
            code: vec![
                "/C".to_string(),
                "shutdown".to_string(),
                "-r".to_string(),
                "-t".to_string(),
                "0".to_string(),
            ],
        }
    }

    pub fn runing(self) -> Result<(), std::io::Error> {
        let _ = Command::new(self.file_path).args(self.code).output()?;
        Ok(())
    }
}

#[tauri::command]
pub fn run_script(file_path: String, code: Vec<String>) -> Result<(), String> {
    use std::path::Path;

    let path = Path::new(&file_path);

    // 如果文件存在，直接执行
    if path.exists() {
        let res = RunCode::new(file_path.clone(), code.clone());
        if let Err(e) = res.runing() {
            return Err(format!("执行文件失败: {} - 错误: {}", file_path, e));
        }
        return Ok(());
    }

    // 将整个命令作为字符串通过cmd执行
    if cfg!(target_os = "windows") {
        // 构建完整的命令字符串
        let mut full_cmd = file_path.clone();
        if !code.is_empty() {
            for arg in &code {
                full_cmd.push(' ');
                full_cmd.push_str(arg);
            }
        }

        println!("通过cmd执行命令: {}", full_cmd);

        // 通过cmd /c执行
        let output = std::process::Command::new("cmd")
            .args(&["/C", &full_cmd])
            .output()
            .map_err(|e| format!("cmd执行失败: {} - 错误: {}", full_cmd, e))?;

        if output.status.success() {
            println!("命令执行成功: {}", full_cmd);
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(format!("命令执行失败: {} - 错误: {}", full_cmd, error_msg))
        }
    } else {
        let res = RunCode::new(file_path.clone(), code.clone());
        if let Err(e) = res.runing() {
            return Err(format!("执行失败: {} {:?} - 错误: {}", file_path, code, e));
        }
        Ok(())
    }
}
