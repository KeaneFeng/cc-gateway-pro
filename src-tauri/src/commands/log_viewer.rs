//! CC-Gateway-Pro: Proxy Request Log Viewer (fork-only module)
//!
//! 将代理请求日志写入每日文件（proxy-YYYY-MM-DD.log），7天自动清理。
//! 独立模块封装，避免上游合并覆盖。

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// 当前日期缓存，避免每次请求都格式化日期
static CURRENT_DATE: Mutex<String> = Mutex::new(String::new());
static CURRENT_DATE_DAY: Mutex<u32> = Mutex::new(0);

/// 获取当前日志目录路径
pub fn get_log_dir() -> PathBuf {
    crate::panic_hook::get_log_dir()
}

/// 获取当前日期对应的代理请求日志文件路径
pub fn get_current_proxy_log_file() -> PathBuf {
    let log_dir = get_log_dir();
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    log_dir.join(format!("proxy-{}.log", today))
}

/// 获取今天的日期字符串（带缓存，每天只格式化一次）
fn get_today_str() -> String {
    let now = chrono::Local::now();
    let day = chrono::Datelike::day(&now);
    {
        let cached_day = CURRENT_DATE_DAY.lock().unwrap();
        if *cached_day == day {
            let cached = CURRENT_DATE.lock().unwrap();
            return cached.clone();
        }
    }
    let today = now.format("%Y-%m-%d").to_string();
    {
        let mut cached_day = CURRENT_DATE_DAY.lock().unwrap();
        let mut cached = CURRENT_DATE.lock().unwrap();
        *cached_day = day;
        *cached = today.clone();
    }
    let cached = CURRENT_DATE.lock().unwrap();
    cached.clone()
}

/// 将代理请求信息追加到当日日志文件
///
/// 格式: [HH:MM:SS] AppType | Provider | Model | Input+Output tokens | Status | Latency
pub fn append_proxy_request_line(line: String) {
    let log_dir = get_log_dir();

    // 确保目录存在
    let _ = std::fs::create_dir_all(&log_dir);

    let today = get_today_str();
    let log_path = log_dir.join(format!("proxy-{}.log", today));

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&log_path) {
        let timestamp = chrono::Local::now().format("%H:%M:%S");
        let _ = writeln!(file, "[{}] {}", timestamp, line);
    }
}

/// 生成代理请求日志行
#[allow(clippy::too_many_arguments)]
pub fn format_proxy_request_line(
    app_type: &str,
    provider_name: &str,
    model: &str,
    request_model: &str,
    input_tokens: u32,
    output_tokens: u32,
    cache_read_tokens: u32,
    status_code: u16,
    latency_ms: u64,
    error_message: Option<&str>,
) -> String {
    let mut line = format!("{} | {} | {}", app_type, provider_name, model);

    // 显示 request_model 映射（如果和 model 不同）
    if request_model != model && !request_model.is_empty() {
        line.push_str(&format!(" -> {}", request_model));
    }

    // tokens
    if input_tokens > 0 || output_tokens > 0 {
        line.push_str(&format!(" | {}+{}", input_tokens, output_tokens));
        if cache_read_tokens > 0 {
            line.push_str(&format!(" (R{})", cache_read_tokens));
        }
    } else {
        line.push_str(" | 0+0");
    }

    // status
    line.push_str(&format!(" | {}", status_code));

    // latency
    if latency_ms > 0 {
        let secs = latency_ms as f64 / 1000.0;
        line.push_str(&format!(" | {:.1}s", secs));
    } else {
        line.push_str(" | 0s");
    }

    // error — 增强显示
    if let Some(err) = error_message {
        let enhanced = classify_error(err);
        line.push_str(&format!(" | ERR: {}", enhanced));
    }

    line
}

/// 增强错误消息显示
///
/// 对常见的 reqwest/reqwest/hyper 错误分类提取详细信息，
/// 否则返回原始消息。
pub fn classify_error(msg: &str) -> String {
    // 连接级错误：剥掉"转发失败: 上游请求失败:"包装
    if msg.contains("client error (Connect)") {
        let detail = extract_connect_detail(msg);
        return format!("连接失败: {}", detail);
    }
    // 转发失败：提取底层错误
    if let Some(inner) = msg.strip_prefix("转发失败: ") {
        return classify_error(inner);
    }
    // 上游请求失败：继续剥
    if let Some(inner) = msg.strip_prefix("上游请求失败: ") {
        return classify_error(inner);
    }
    // 超时错误
    if msg.contains("Timeout") || msg.contains("timeout") || msg.contains("超时") {
        return "请求超时".to_string();
    }
    // 上游 HTTP 错误
    if msg.contains("上游错误 (") {
        return msg.to_string();
    }
    // 熔断
    if msg.contains("熔断") {
        return msg.to_string();
    }
    // 默认返回原始消息
    msg.to_string()
}

/// 从 Connect 错误中提取连接细节
fn extract_connect_detail(msg: &str) -> String {
    // 常见模式:
    // "client error (Connect): connection refused" → connection refused
    // "client error (Connect): dns error" → dns error
    // "client error (Connect): tcp connect error: ..." → tcp connect error: ...
    if let Some(pos) = msg.find("client error (Connect): ") {
        let detail = &msg[pos + "client error (Connect): ".len()..];
        // 截取前 80 字符
        let truncated = detail.chars().take(80).collect::<String>();
        return truncated;
    }
    // fallback: 取 "): " 之后的内容
    if let Some(pos) = msg.find("): ") {
        let detail = &msg[pos + 3..];
        return detail.chars().take(80).collect();
    }
    msg.to_string()
}

/// 启动时清理 7 天前的代理请求日志文件
pub fn cleanup_old_logs(retention_days: u64) {
    let log_dir = get_log_dir();
    if !log_dir.exists() {
        return;
    }

    let cutoff = chrono::Local::now()
        .checked_sub_days(chrono::Days::new(retention_days))
        .map(|dt| dt.naive_local().date())
        .unwrap_or_else(|| chrono::Local::now().naive_local().date());

    let Ok(entries) = std::fs::read_dir(&log_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        // 匹配 proxy-YYYY-MM-DD.log 格式
        let date_str = file_name
            .strip_prefix("proxy-")
            .and_then(|s| s.strip_suffix(".log"))
            .filter(|s| s.len() == 10 && s.chars().all(|c| c.is_ascii_digit() || c == '-'));

        let Some(date_str) = date_str else {
            continue;
        };

        if let Ok(file_date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            if file_date < cutoff {
                let _ = std::fs::remove_file(&path);
                log::info!("[LogCleaner] 清理过期代理日志: {}", file_name);
            }
        }
    }
}

/// 在用户首选终端中打开代理请求日志，使用 tail -f 实时查看
#[tauri::command]
pub fn open_log_viewer() -> Result<bool, String> {
    let log_file = get_current_proxy_log_file();

    // 如果今天的日志文件还不存在，尝试找最近的日志文件
    let log_path = if log_file.exists() {
        log_file
    } else {
        let log_dir = get_log_dir();
        // 查找最近的 proxy log 文件
        let mut latest: Option<PathBuf> = None;
        let mut latest_mtime: std::time::SystemTime = std::time::UNIX_EPOCH;

        if let Ok(entries) = std::fs::read_dir(&log_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("proxy-") && name.ends_with(".log") {
                        if let Ok(meta) = entry.metadata() {
                            if let Ok(mtime) = meta.modified() {
                                if mtime > latest_mtime {
                                    latest_mtime = mtime;
                                    latest = Some(path);
                                }
                            }
                        }
                    }
                }
            }
        }

        latest.ok_or_else(|| "代理日志文件不存在，请先使用代理生成请求日志".to_string())?
    };

    let log_path_str = log_path.to_string_lossy().to_string();
    launch_tail_terminal(&log_path_str)
}

/// 在首选终端中启动 tail -f 查看日志
fn launch_tail_terminal(log_path: &str) -> Result<bool, String> {
    let preferred = crate::settings::get_preferred_terminal();
    let terminal = preferred.as_deref().unwrap_or("terminal");

    let temp_dir = std::env::temp_dir();
    let pid = std::process::id();
    let script_file = temp_dir.join(format!("cc_gateway_log_viewer_{}.sh", pid));

    let script_content = format!(
        r#"#!/bin/bash
trap 'rm -f "{script_path}"' EXIT
echo "=== CC Gateway Pro 代理请求日志 ==="
echo "文件: {log_path}"
echo "按 Ctrl+C 停止查看"
echo ""
tail -f "{log_path}"
"#,
        script_path = script_file.display(),
        log_path = log_path,
    );

    std::fs::write(&script_file, &script_content).map_err(|e| format!("写入脚本失败: {e}"))?;

    #[cfg(target_os = "macos")]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&script_file, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("设置权限失败: {e}"))?;
        launch_macos_terminal(terminal, &script_file)
    }

    #[cfg(target_os = "linux")]
    {
        launch_linux_terminal(terminal, &script_file)
    }

    #[cfg(target_os = "windows")]
    {
        launch_windows_terminal(&script_file)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err("不支持的操作系统".to_string())
    }
}

#[cfg(target_os = "macos")]
fn launch_macos_terminal(terminal: &str, script_file: &std::path::Path) -> Result<bool, String> {
    let script_path = script_file.display().to_string();

    let result = match terminal {
        "iterm2" => {
            use std::process::Command;
            let applescript = format!(
                r#"tell application "iTerm2"
    activate
    tell current window
        create tab with default profile
        tell current session
            write text "bash '{script_path}'"
        end tell
    end tell
end tell"#,
                script_path = script_path
            );
            Command::new("osascript")
                .arg("-e")
                .arg(&applescript)
                .output()
        }
        "warp" => {
            use std::process::Command;
            let mut cmd = Command::new("open");
            cmd.arg(format!(
                "warp://action/new_tab?path={}",
                script_file.to_string_lossy()
            ));
            cmd.output()
        }
        "kitty" | "alacritty" | "ghostty" | "wezterm" | "kaku" => {
            use std::process::Command;
            let app_name = match terminal {
                "kitty" => "kitty",
                "alacritty" => "Alacritty",
                "ghostty" => "Ghostty",
                "wezterm" => "WezTerm",
                "kaku" => "Kaku",
                _ => terminal,
            };
            Command::new("open")
                .arg("-a")
                .arg(app_name)
                .arg("--args")
                .arg("bash")
                .arg(&script_path)
                .output()
        }
        _ => {
            use std::process::Command;
            let applescript = format!(
                r#"tell application "Terminal"
    activate
    do script "bash '{script_path}'"
end tell"#,
                script_path = script_path
            );
            Command::new("osascript")
                .arg("-e")
                .arg(&applescript)
                .output()
        }
    };

    result.map_err(|e| format!("启动终端失败: {e}"))?;
    Ok(true)
}

#[cfg(target_os = "linux")]
fn launch_linux_terminal(terminal: &str, script_file: &std::path::Path) -> Result<bool, String> {
    use std::process::Command;

    let terminals = match terminal {
        "gnome-terminal" => vec![("gnome-terminal", vec!["--", "bash"])],
        "konsole" => vec![("konsole", vec!["-e", "bash"])],
        "kitty" => vec![("kitty", vec!["bash"])],
        "alacritty" => vec![("alacritty", vec!["-e", "bash"])],
        "ghostty" => vec![("ghostty", vec!["--", "bash"])],
        _ => vec![
            ("gnome-terminal", vec!["--", "bash"]),
            ("konsole", vec!["-e", "bash"]),
            ("kitty", vec!["bash"]),
        ],
    };

    let script_path = script_file.to_string_lossy();

    for (term, args) in terminals {
        let result = Command::new(term)
            .args(&args)
            .arg(script_path.as_ref())
            .spawn();
        if result.is_ok() {
            return Ok(true);
        }
    }

    Err("未找到可用的终端".to_string())
}

#[cfg(target_os = "windows")]
fn launch_windows_terminal(script_file: &std::path::Path) -> Result<bool, String> {
    use std::process::Command;

    let script_path = script_file.to_string_lossy();
    let result = Command::new("cmd")
        .args(["/c", "start", "cmd", "/k", &format!("bash {}", script_path)])
        .spawn();

    result.map_err(|e| format!("启动终端失败: {e}"))?;
    Ok(true)
}
