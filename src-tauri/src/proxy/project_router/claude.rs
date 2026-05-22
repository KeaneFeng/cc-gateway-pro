//! Claude Session Project Scanner
//!
//! 扫描 ~/.claude/projects/ 目录下的 JSONL 会话文件，
//! 提取 session_id → project cwd 映射。

use super::scanner::{SessionProjectMap, SessionProjectScanner};
use std::path::PathBuf;

pub struct ClaudeScanner;

impl ClaudeScanner {
    fn projects_root() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".claude")
            .join("projects")
    }
}

impl SessionProjectScanner for ClaudeScanner {
    fn app_type(&self) -> &'static str {
        "claude"
    }

    fn scan_all(&self) -> SessionProjectMap {
        let mut discovered = SessionProjectMap::new();
        let projects_dir = Self::projects_root();
        if !projects_dir.exists() {
            return discovered;
        }

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
                                    if let Ok(json) =
                                        serde_json::from_str::<serde_json::Value>(line)
                                    {
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

        discovered
    }

    fn scan_one(&self, session_id: &str) -> Option<String> {
        let projects_dir = Self::projects_root();
        if !projects_dir.exists() {
            return None;
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

        found_cwd
    }

    fn list_project_paths(&self) -> Vec<String> {
        let projects_dir = Self::projects_root();
        if !projects_dir.exists() {
            return Vec::new();
        }

        let mut project_paths: Vec<String> = Vec::new();

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
                                    if let Ok(json) =
                                        serde_json::from_str::<serde_json::Value>(line)
                                    {
                                        if let Some(cwd) = json.get("cwd").and_then(|v| v.as_str())
                                        {
                                            if !cwd.is_empty()
                                                && !project_paths.contains(&cwd.to_string())
                                            {
                                                project_paths.push(cwd.to_string());
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

        project_paths
    }
}
