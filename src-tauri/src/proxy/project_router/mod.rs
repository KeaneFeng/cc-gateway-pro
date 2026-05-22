//! Project Router Module
//!
//! 提供通用的 session → project → provider 路由逻辑。
//! 不同 app（Claude / Codex）通过实现 `SessionProjectScanner` trait 接入。

pub mod claude;
pub mod codex;
pub mod scanner; // placeholder for Task 2

use crate::database::Database;
use scanner::SessionProjectScanner;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 通用的 Project Router
///
/// 持有一个 Scanner 实现，负责扫描 session → project 映射，
/// 并通过 DB settings 表查找 project → provider 绑定。
#[allow(dead_code)]
pub struct ProjectRouter {
    /// 具体的 scanner 实现（Claude / Codex）
    scanner: Box<dyn SessionProjectScanner>,
    /// 缓存：session_id → project cwd
    cache: RwLock<HashMap<String, String>>,
    /// Database reference
    db: Arc<Database>,
    /// DB settings key，如 "project_providers" (claude) 或 "project_providers_codex"
    settings_key: &'static str,
}

#[allow(dead_code)]
impl ProjectRouter {
    /// 创建 Claude 专用 router
    ///
    /// DB key 使用 "project_providers"（保持与现有配置兼容）
    pub fn new_claude(db: Arc<Database>) -> Self {
        Self {
            scanner: Box::new(claude::ClaudeScanner),
            cache: RwLock::new(HashMap::new()),
            db,
            settings_key: "project_providers",
        }
    }

    /// 创建 Codex 专用 router
    ///
    /// DB key 使用 "project_providers_codex"（与 claude 隔离）
    pub fn new_codex(db: Arc<Database>) -> Self {
        Self {
            scanner: Box::new(codex::CodexScanner),
            cache: RwLock::new(HashMap::new()),
            db,
            settings_key: "project_providers_codex",
        }
    }

    /// 全量扫描：app 启动时调用
    pub fn scan_projects(&self) {
        let discovered = self.scanner.scan_all();
        let count = discovered.len();
        if let Ok(mut map) = self.cache.write() {
            for (sid, cwd) in discovered {
                map.entry(sid).or_insert(cwd);
            }
            log::info!(
                "🗺️ [{}] ProjectRouter: loaded {} session→project mappings",
                self.scanner.app_type(),
                map.len()
            );
        }
        if count > 0 {
            log::info!(
                "🗺️ [{}] Scanned {} session mappings",
                self.scanner.app_type(),
                count
            );
        }
    }

    /// 查找 session 对应的 provider_id
    ///
    /// 流程：session_id → project cwd → DB settings → provider_id
    pub fn get_provider_for_session(&self, session_id: &str) -> Option<String> {
        // 1. 从缓存查找
        let project_path = {
            let map = self.cache.read().ok()?;
            map.get(session_id).cloned()
        };

        // 2. 缓存未命中：增量扫描
        let project_path = match project_path {
            Some(p) => p,
            None => {
                if let Some(cwd) = self.scanner.scan_one(session_id) {
                    if let Ok(mut map) = self.cache.write() {
                        map.entry(session_id.to_string())
                            .or_insert_with(|| cwd.clone());
                    }
                    cwd
                } else {
                    return None;
                }
            }
        };

        log::info!(
            "[ProjectRouter/{}] session {} -> project {}",
            self.scanner.app_type(),
            session_id,
            project_path
        );

        // 3. 从 DB settings 查找 provider_id
        let project_providers: HashMap<String, String> =
            match self.db.get_setting(self.settings_key) {
                Ok(Some(json_str)) => serde_json::from_str(&json_str).unwrap_or_default(),
                _ => {
                    log::warn!(
                        "[ProjectRouter/{}] No {} in DB settings!",
                        self.scanner.app_type(),
                        self.settings_key
                    );
                    return None;
                }
            };

        log::info!(
            "[ProjectRouter/{}] DB {}: {} entries",
            self.scanner.app_type(),
            self.settings_key,
            project_providers.len()
        );

        // 4. 三段查找：direct match → canonical → prefix
        if let Some(provider_id) = project_providers.get(&project_path) {
            log::info!(
                "[ProjectRouter/{}] Direct match: {} -> {}",
                self.scanner.app_type(),
                project_path,
                provider_id
            );
            return Some(provider_id.clone());
        }

        if let Ok(canonical) = std::fs::canonicalize(&project_path) {
            let canon_str = canonical.to_string_lossy().to_string();
            if let Some(provider_id) = project_providers.get(&canon_str) {
                log::info!(
                    "[ProjectRouter/{}] Canonical match: {} -> {}",
                    self.scanner.app_type(),
                    canon_str,
                    provider_id
                );
                return Some(provider_id.clone());
            }
        }

        for (proj, provider_id) in &project_providers {
            if project_path.starts_with(proj.as_str()) || proj.starts_with(project_path.as_str()) {
                log::info!(
                    "[ProjectRouter/{}] Prefix match: {} <-> {} -> {}",
                    self.scanner.app_type(),
                    project_path,
                    proj,
                    provider_id
                );
                return Some(provider_id.clone());
            }
        }

        log::warn!(
            "[ProjectRouter/{}] No match for project: {}",
            self.scanner.app_type(),
            project_path
        );
        None
    }

    /// 获取 session 对应的 project cwd
    pub fn get_project_for_session(&self, session_id: &str) -> Option<String> {
        self.cache
            .read()
            .ok()
            .and_then(|map| map.get(session_id).cloned())
    }
}
