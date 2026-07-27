//! Panic Hook 模块
//!
//! 在应用崩溃时捕获 panic 信息并记录到 `<app_config_dir>/crash.log` 文件中（默认 `~/.cc-gateway-pro/crash.log`）。
//! 便于用户和开发者诊断闪退问题。

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::panic;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

/// 应用版本号（从 Cargo.toml 读取）
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const CRASH_LOG_MAX_SIZE: u64 = 5 * 1024 * 1024;
const CRASH_LOG_ARCHIVES_TO_KEEP: usize = 2;

static APP_CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();
static CRASH_LOG_LOCK: Mutex<()> = Mutex::new(());

pub fn init_app_config_dir(dir: PathBuf) {
    let _ = APP_CONFIG_DIR.set(dir);
}

/// Ensure application-owned directories are private on Unix.
pub fn ensure_private_dir(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

/// Ensure files containing credentials or diagnostics are private on Unix.
pub fn ensure_private_file(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

/// Harden already-created files in an application-owned directory.
pub fn ensure_private_files_in_dir(path: &Path) -> std::io::Result<()> {
    ensure_private_dir(path)?;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            ensure_private_file(&entry.path())?;
        }
    }
    Ok(())
}

/// 获取默认应用配置目录（不会 panic）
fn default_app_config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cc-gateway-pro")
}

/// 获取应用配置目录（优先使用初始化时写入的值；不会 panic）
fn get_app_config_dir() -> PathBuf {
    APP_CONFIG_DIR
        .get()
        .cloned()
        .unwrap_or_else(default_app_config_dir)
}

/// 获取崩溃日志文件路径
fn get_crash_log_path() -> PathBuf {
    get_app_config_dir().join("crash.log")
}

fn rotated_crash_log_path(path: &Path, index: usize) -> PathBuf {
    let mut rotated = path.as_os_str().to_os_string();
    rotated.push(format!(".{index}"));
    PathBuf::from(rotated)
}

fn rotate_crash_log_if_needed_with_limit(
    path: &Path,
    max_size: u64,
    archives_to_keep: usize,
) -> std::io::Result<()> {
    let size = match fs::metadata(path) {
        Ok(metadata) => metadata.len(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e),
    };
    if size < max_size || archives_to_keep == 0 {
        return Ok(());
    }

    for index in (1..=archives_to_keep).rev() {
        let source = if index == 1 {
            path.to_path_buf()
        } else {
            rotated_crash_log_path(path, index - 1)
        };
        if !source.exists() {
            continue;
        }

        let destination = rotated_crash_log_path(path, index);
        if destination.exists() {
            fs::remove_file(&destination)?;
        }
        fs::rename(source, destination)?;
    }

    Ok(())
}

fn rotate_crash_log_if_needed(path: &Path) -> std::io::Result<()> {
    rotate_crash_log_if_needed_with_limit(path, CRASH_LOG_MAX_SIZE, CRASH_LOG_ARCHIVES_TO_KEEP)
}

fn redact_panic_message(message: &str) -> String {
    crate::redact_sensitive_text_for_storage(message)
}

/// 获取日志目录路径
pub fn get_log_dir() -> PathBuf {
    get_app_config_dir().join("logs")
}

/// 安全获取环境信息（不会 panic）
fn get_system_info() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let family = std::env::consts::FAMILY;

    // 安全获取当前工作目录
    let cwd = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // 安全获取当前线程信息
    let thread = std::thread::current();
    let thread_name = thread.name().unwrap_or("unnamed");
    let thread_id = format!("{:?}", thread.id());

    format!(
        "OS: {os} ({family})\n\
         Arch: {arch}\n\
         App Version: {APP_VERSION}\n\
         Working Dir: {cwd}\n\
         Thread: {thread_name} (ID: {thread_id})"
    )
}

/// 设置 panic hook，捕获崩溃信息并写入日志文件
///
/// 在应用启动时调用此函数，确保任何 panic 都会被记录。
/// 日志格式包含：
/// - 时间戳
/// - 应用版本和系统信息
/// - Panic 信息
/// - 发生位置（文件:行号）
/// - Backtrace（完整调用栈）
pub fn setup_panic_hook() {
    // 启用 backtrace（确保 release 模式也能捕获）
    if std::env::var("RUST_BACKTRACE").is_err() {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    let default_hook = panic::take_hook();

    panic::set_hook(Box::new(move |panic_info| {
        let log_path = get_crash_log_path();

        // 确保目录存在
        if let Some(parent) = log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
            let _ = ensure_private_dir(parent);
        }

        // 构建崩溃信息（使用 catch_unwind 保护时间格式化，避免嵌套 panic）
        let timestamp = std::panic::catch_unwind(|| {
            chrono::Local::now()
                .format("%Y-%m-%d %H:%M:%S%.3f")
                .to_string()
        })
        .unwrap_or_else(|_| {
            // chrono panic 时回退到 unix timestamp
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| format!("unix:{}.{:03}", d.as_secs(), d.subsec_millis()))
                .unwrap_or_else(|_| "unknown".to_string())
        });

        // 获取系统信息
        let system_info = std::panic::catch_unwind(get_system_info)
            .unwrap_or_else(|_| "Failed to get system info".to_string());

        // 获取 panic 消息（尝试多种方式提取）
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            // 尝试使用 Display trait
            format!("{panic_info}")
        };
        let message = redact_panic_message(&message);

        // 获取位置信息
        let location = if let Some(loc) = panic_info.location() {
            format!(
                "File: {}\n         Line: {}\n         Column: {}",
                loc.file(),
                loc.line(),
                loc.column()
            )
        } else {
            "Unknown location".to_string()
        };

        // 捕获 backtrace（完整调用栈）
        let backtrace = std::backtrace::Backtrace::force_capture();
        let backtrace_str = format!("{backtrace}");

        // 格式化日志条目
        let separator = "=".repeat(80);
        let sub_separator = "-".repeat(40);
        let crash_entry = format!(
            r#"
{separator}
[CRASH REPORT] {timestamp}
{separator}

{sub_separator}
System Information
{sub_separator}
{system_info}

{sub_separator}
Error Details
{sub_separator}
Message: {message}

Location: {location}

{sub_separator}
Stack Trace (Backtrace)
{sub_separator}
{backtrace_str}

{separator}
"#
        );

        // 将 size check、轮转和追加合成同一个临界区，避免多线程同时 panic
        // 时两个 hook 竞争 rename 而丢失归档。
        let crash_log_guard = CRASH_LOG_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _ = rotate_crash_log_if_needed(&log_path);
        let saved =
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&log_path) {
                let _ = ensure_private_file(&log_path);
                let _ = file.write_all(crash_entry.as_bytes());
                let _ = file.flush();
                true
            } else {
                false
            };
        drop(crash_log_guard);

        if saved {
            eprintln!(
                "\n[CC Gateway Pro] Crash log saved to: {}",
                log_path.display()
            );
        }

        // 同时输出到 stderr（便于开发调试）
        eprintln!("{crash_entry}");

        // 调用默认 hook
        default_hook(panic_info);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crash_log_path() {
        let path = get_crash_log_path();
        assert!(path.ends_with("crash.log"));
        assert!(path.to_string_lossy().contains(".cc-gateway-pro"));
    }

    #[test]
    fn test_system_info() {
        let info = get_system_info();
        assert!(info.contains("OS:"));
        assert!(info.contains("Arch:"));
        assert!(info.contains("App Version:"));
    }

    #[test]
    fn crash_log_rotation_keeps_bounded_archives() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("crash.log");

        fs::write(&path, b"first").unwrap();
        rotate_crash_log_if_needed_with_limit(&path, 4, 2).unwrap();
        assert!(!path.exists());
        assert_eq!(
            fs::read(rotated_crash_log_path(&path, 1)).unwrap(),
            b"first"
        );

        fs::write(&path, b"second").unwrap();
        rotate_crash_log_if_needed_with_limit(&path, 4, 2).unwrap();
        assert_eq!(
            fs::read(rotated_crash_log_path(&path, 1)).unwrap(),
            b"second"
        );
        assert_eq!(
            fs::read(rotated_crash_log_path(&path, 2)).unwrap(),
            b"first"
        );

        fs::write(&path, b"third").unwrap();
        rotate_crash_log_if_needed_with_limit(&path, 4, 2).unwrap();
        assert_eq!(
            fs::read(rotated_crash_log_path(&path, 1)).unwrap(),
            b"third"
        );
        assert_eq!(
            fs::read(rotated_crash_log_path(&path, 2)).unwrap(),
            b"second"
        );
        assert!(!rotated_crash_log_path(&path, 3).exists());
    }

    #[cfg(unix)]
    #[test]
    fn private_permissions_are_enforced() {
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("private");
        fs::create_dir(&dir).unwrap();
        let file = dir.join("secret.db");
        fs::write(&file, b"secret").unwrap();

        ensure_private_dir(&dir).unwrap();
        ensure_private_file(&file).unwrap();

        assert_eq!(
            fs::metadata(&dir).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(&file).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    #[test]
    fn panic_message_redaction_masks_credentials_without_touching_diagnostics() {
        let message = concat!(
            "request failed: Bearer sk-live-1234567890 ",
            "api_key=xai-secret-token-12345 ",
            "url=https://example.test/v1?token=query-token-123"
        );
        let redacted = redact_panic_message(message);

        assert!(redacted.starts_with("request failed:"));
        assert!(!redacted.contains("sk-live-1234567890"));
        assert!(!redacted.contains("xai-secret-token-12345"));
        assert!(!redacted.contains("query-token-123"));
        assert!(redacted.matches("[REDACTED]").count() >= 3);
    }
}
