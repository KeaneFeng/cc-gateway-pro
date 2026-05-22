//! Project Router Scanner Trait
//!
//! 定义 session → project 路径扫描器的抽象接口。
//! 不同 app（Claude / Codex / 未来）实现各自的扫描逻辑。

use std::collections::HashMap;

/// 扫描结果：session_id -> project cwd
pub type SessionProjectMap = HashMap<String, String>;

/// 不同 app 的 session→project 扫描器 trait
///
/// 实现此 trait 即可接入 ProjectRouter 的通用路由逻辑。
pub trait SessionProjectScanner: Send + Sync {
    /// app 类型字符串（用于日志和 DB key 后缀）
    fn app_type(&self) -> &'static str;

    /// 全量扫描：app 启动时调用一次，建立缓存
    fn scan_all(&self) -> SessionProjectMap;

    /// 增量扫描：缓存未命中时调用，尽量按 session_id 快速定位单文件
    fn scan_one(&self, session_id: &str) -> Option<String>;

    /// 列出已知的所有 project cwd（用于前端 ProjectRoutingPage）
    fn list_project_paths(&self) -> Vec<String>;
}
