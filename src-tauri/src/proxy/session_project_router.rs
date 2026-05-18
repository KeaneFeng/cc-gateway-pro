//! Session Project Router
//!
//! Scans ~/.claude/projects/ JSONL files to build session_id → project_path mapping.
//! This enables project-level provider routing: different projects can use different providers.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

/// Maps session_id → project_path (e.g., "abc-123" → "/Users/keane/www/apd")
#[derive(Debug, Default)]
pub struct SessionProjectRouter {
    /// session_id → project directory path
    session_projects: RwLock<HashMap<String, String>>,
    /// project_path → provider_id (from provider meta configuration)
    project_providers: RwLock<HashMap<String, String>>,
    /// Project directories to scan (from provider meta configuration)
    project_dirs: RwLock<Vec<String>>,
}

impl SessionProjectRouter {
    pub fn new() -> Self {
        Self::default()
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

    /// Update project_providers mapping (called when provider meta changes)
    pub fn update_project_providers(&self, mapping: HashMap<String, String>) {
        if let Ok(mut map) = self.project_providers.write() {
            *map = mapping;
            log::info!("🗺️ Updated project_providers: {} entries", map.len());
        }
    }

    /// Look up the provider_id for a given session_id
    pub fn get_provider_for_session(&self, session_id: &str) -> Option<String> {
        let session_projects = self.session_projects.read().ok()?;
        let project_path = session_projects.get(session_id)?;

        let project_providers = self.project_providers.read().ok()?;
        // Try canonical path first
        if let Some(provider_id) = project_providers.get(project_path) {
            return Some(provider_id.clone());
        }
        // Try canonicalizing
        if let Ok(canonical) = std::fs::canonicalize(project_path) {
            let canon_str = canonical.to_string_lossy().to_string();
            if let Some(provider_id) = project_providers.get(&canon_str) {
                return Some(provider_id.clone());
            }
        }
        // Try prefix matching (for paths that are subdirectories)
        for (proj, provider_id) in project_providers.iter() {
            if project_path.starts_with(proj) || proj.starts_with(project_path) {
                return Some(provider_id.clone());
            }
        }
        None
    }

    /// Get the project path for a session_id
    pub fn get_project_for_session(&self, session_id: &str) -> Option<String> {
        self.session_projects
            .read()
            .ok()
            .and_then(|map| map.get(session_id).cloned())
    }
}
