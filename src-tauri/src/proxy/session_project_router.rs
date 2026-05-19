//! Session Project Router
//!
//! Scans ~/.claude/projects/ JSONL files to build session_id → project_path mapping.
//! This enables project-level provider routing: different projects can use different providers.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::database::Database;

/// Maps session_id → project_path (e.g., "abc-123" → "/Users/keane/www/apd")
pub struct SessionProjectRouter {
    /// session_id → project directory path
    session_projects: RwLock<HashMap<String, String>>,
    /// Database reference for reading project_providers from settings table
    db: Arc<Database>,
}

impl SessionProjectRouter {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            session_projects: RwLock::new(HashMap::new()),
            db,
        }
    }

    /// Scan ~/.claude/projects/ JSONL files to build session → project mapping
    pub fn scan_projects(&self) {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let projects_dir = home.join(".claude").join("projects");
        if !projects_dir.exists() {
            log::debug!("No ~/.claude/projects/ directory found, skipping session scan");
            return;
        }

        let mut discovered: HashMap<String, String> = HashMap::new();

        // 目录结构: ~/.claude/projects/<encoded-path>/<session-id>.jsonl
        if let Ok(entries) = std::fs::read_dir(&projects_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                if let Ok(files) = std::fs::read_dir(&path) {
                    for file in files.flatten() {
                        let fpath = file.path();
                        if fpath.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                            if let Ok(content) = std::fs::read_to_string(&fpath) {
                                for line in content.lines() {
                                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                                        if let (Some(sid), Some(cwd)) = (
                                            json.get("sessionId").and_then(|v| v.as_str()),
                                            json.get("cwd").and_then(|v| v.as_str()),
                                        ) {
                                            // 只在 cwd 非空时插入（permission-mode 行有 sessionId 但无 cwd）
                                            if !cwd.is_empty() {
                                                discovered
                                                    .entry(sid.to_string())
                                                    .or_insert_with(|| cwd.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let count = discovered.len();
        if let Ok(mut map) = self.session_projects.write() {
            for (sid, cwd) in discovered {
                map.entry(sid).or_insert(cwd);
            }
            log::info!("🗺️ SessionProjectRouter: loaded {} session→project mappings", map.len());
        }
        if count > 0 {
            log::info!("🗺️ Scanned {} session mappings from JSONL files", count);
        }
    }

    /// Look up the provider_id for a given session_id
    /// Reads project_providers from DB settings table (same storage as UI)
    pub fn get_provider_for_session(&self, session_id: &str) -> Option<String> {
        // First check cache
        let project_path = {
            let map = self.session_projects.read().ok()?;
            map.get(session_id).cloned()
        };

        let project_path = match project_path {
            Some(p) => p,
            None => {
                // Unknown session: try incremental scan (for sessions created after app startup)
                self.scan_session_incremental(session_id);
                let map = self.session_projects.read().ok()?;
                map.get(session_id).cloned()?
            }
        };

        log::info!("[ProjectRouter] session {} -> project {}", session_id, project_path);

        // 从 DB settings 表读取 project_providers（和 UI 共享同一份数据）
        let project_providers: HashMap<String, String> = match self.db.get_setting("project_providers") {
            Ok(Some(json_str)) => {
                let pp: HashMap<String, String> = serde_json::from_str(&json_str).unwrap_or_default();
                log::info!("[ProjectRouter] DB project_providers: {} entries", pp.len());
                for (k, v) in &pp {
                    log::info!("[ProjectRouter]   {} -> {}", k, v);
                }
                pp
            }
            Ok(None) => {
                log::warn!("[ProjectRouter] No project_providers in DB settings!");
                return None;
            }
            Err(e) => {
                log::error!("[ProjectRouter] DB error reading project_providers: {}", e);
                return None;
            }
        };

        // Try canonical path first
        if let Some(provider_id) = project_providers.get(&project_path) {
            log::info!("[ProjectRouter] Direct match: {} -> {}", project_path, provider_id);
            return Some(provider_id.clone());
        }
        // Try canonicalizing
        if let Ok(canonical) = std::fs::canonicalize(&project_path) {
            let canon_str = canonical.to_string_lossy().to_string();
            if let Some(provider_id) = project_providers.get(&canon_str) {
                log::info!("[ProjectRouter] Canonical match: {} -> {}", canon_str, provider_id);
                return Some(provider_id.clone());
            }
        }
        // Try prefix matching
        for (proj, provider_id) in &project_providers {
            if project_path.starts_with(proj.as_str()) || proj.starts_with(project_path.as_str()) {
                log::info!("[ProjectRouter] Prefix match: {} <-> {} -> {}", project_path, proj, provider_id);
                return Some(provider_id.clone());
            }
        }
        log::warn!("[ProjectRouter] No match for project: {}", project_path);
        None
    }

    /// Get the project path for a session_id
    pub fn get_project_for_session(&self, session_id: &str) -> Option<String> {
        self.session_projects
            .read()
            .ok()
            .and_then(|map| map.get(session_id).cloned())
    }

    /// Incremental scan: find the JSONL file containing this session_id
    /// Used when a session is not in the startup cache (created after app started)
    fn scan_session_incremental(&self, session_id: &str) {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let projects_dir = home.join(".claude").join("projects");
        if !projects_dir.exists() {
            return;
        }

        let mut found_cwd: Option<String> = None;

        if let Ok(entries) = std::fs::read_dir(&projects_dir) {
            'outer: for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                if let Ok(files) = std::fs::read_dir(&path) {
                    for file in files.flatten() {
                        let fpath = file.path();
                        if fpath.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                            if let Ok(content) = std::fs::read_to_string(&fpath) {
                                for line in content.lines() {
                                    if let Ok(json) =
                                        serde_json::from_str::<serde_json::Value>(line)
                                    {
                                        if let (Some(sid), Some(cwd)) = (
                                            json.get("sessionId").and_then(|v| v.as_str()),
                                            json.get("cwd").and_then(|v| v.as_str()),
                                        ) {
                                            if sid == session_id && !cwd.is_empty() {
                                                found_cwd = Some(cwd.to_string());
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            if found_cwd.is_some() {
                                break;
                            }
                        }
                    }
                }
                if found_cwd.is_some() {
                    break 'outer;
                }
            }
        }

        if let Some(cwd) = found_cwd {
            log::info!(
                "🗺️ SessionProjectRouter: discovered new session {} → {}",
                session_id,
                cwd
            );
            if let Ok(mut map) = self.session_projects.write() {
                map.entry(session_id.to_string()).or_insert(cwd);
            }
        }
    }
}
