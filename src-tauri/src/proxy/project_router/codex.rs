//! Codex Session Project Scanner
//!
//! 扫描 ~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl 文件，
//! 提取 session_id → project cwd 映射。
//!
//! Codex 文件名包含 session_id：rollout-<timestamp>-<uuid>.jsonl

use super::scanner::{SessionProjectMap, SessionProjectScanner};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub struct CodexScanner;

impl CodexScanner {
    fn sessions_root() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".codex")
            .join("sessions")
    }

    /// 从文件名提取 session_id
    /// rollout-2026-05-20T10-22-16-019e4330-dd74-7a90-9eb0-158a7bad2183.jsonl
    /// → "019e4330-dd74-7a90-9eb0-158a7bad2183"
    pub fn extract_session_id_from_filename(filename: &str) -> Option<String> {
        let stem = filename.strip_suffix(".jsonl")?;
        let stem = stem.strip_prefix("rollout-")?;
        // UUID 是最后 5 个 `-` 分隔的段（8-4-4-4-12）
        let parts: Vec<&str> = stem.split('-').collect();
        if parts.len() < 5 {
            return None;
        }
        let uuid_parts = &parts[parts.len() - 5..];
        // 验证 UUID 形状：8-4-4-4-12
        if uuid_parts[0].len() == 8
            && uuid_parts[1].len() == 4
            && uuid_parts[2].len() == 4
            && uuid_parts[3].len() == 4
            && uuid_parts[4].len() == 12
        {
            Some(uuid_parts.join("-"))
        } else {
            None
        }
    }

    /// 从单个 jsonl 首行 session_meta 提取 cwd
    fn extract_cwd_from_file(path: &Path) -> Option<String> {
        let content = std::fs::read_to_string(path).ok()?;
        for line in content.lines().take(5) {
            if !line.contains("\"session_meta\"") {
                continue;
            }
            let v: serde_json::Value = serde_json::from_str(line).ok()?;
            if v.get("type").and_then(|t| t.as_str()) != Some("session_meta") {
                continue;
            }
            let cwd = v.get("payload")?.get("cwd")?.as_str()?;
            if !cwd.is_empty() {
                return Some(cwd.to_string());
            }
        }
        None
    }

    /// 遍历 ~/.codex/sessions/YYYY/MM/DD/ 所有 rollout-*.jsonl
    fn iter_rollout_files(root: &Path) -> Vec<PathBuf> {
        let mut out = Vec::new();
        // 三层目录：YYYY / MM / DD
        let Ok(years) = std::fs::read_dir(root) else {
            return out;
        };
        for y in years.flatten() {
            if !y.path().is_dir() {
                continue;
            }
            let Ok(months) = std::fs::read_dir(y.path()) else {
                continue;
            };
            for m in months.flatten() {
                if !m.path().is_dir() {
                    continue;
                }
                let Ok(days) = std::fs::read_dir(m.path()) else {
                    continue;
                };
                for d in days.flatten() {
                    if !d.path().is_dir() {
                        continue;
                    }
                    let Ok(files) = std::fs::read_dir(d.path()) else {
                        continue;
                    };
                    for f in files.flatten() {
                        let p = f.path();
                        if p.extension().and_then(|e| e.to_str()) == Some("jsonl")
                            && p.file_name()
                                .and_then(|n| n.to_str())
                                .map(|n| n.starts_with("rollout-"))
                                .unwrap_or(false)
                        {
                            out.push(p);
                        }
                    }
                }
            }
        }
        out
    }
}

impl SessionProjectScanner for CodexScanner {
    fn app_type(&self) -> &'static str {
        "codex"
    }

    fn scan_all(&self) -> SessionProjectMap {
        let mut map = SessionProjectMap::new();
        let root = Self::sessions_root();
        if !root.exists() {
            return map;
        }

        for path in Self::iter_rollout_files(&root) {
            let Some(filename) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let Some(sid) = Self::extract_session_id_from_filename(filename) else {
                continue;
            };
            if map.contains_key(&sid) {
                continue;
            }
            if let Some(cwd) = Self::extract_cwd_from_file(&path) {
                // 加 "codex_" 前缀，与 session.rs 的 extract_codex_session 一致
                map.insert(format!("codex_{sid}"), cwd);
            }
        }
        map
    }

    fn scan_one(&self, session_id: &str) -> Option<String> {
        let root = Self::sessions_root();
        if !root.exists() {
            return None;
        }

        // session_id 可能带 "codex_" 前缀，提取原始 UUID
        let raw_session_id = session_id.strip_prefix("codex_").unwrap_or(session_id);

        // 遍历所有文件，匹配文件名含原始 UUID
        for path in Self::iter_rollout_files(&root) {
            let Some(filename) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !filename.contains(raw_session_id) {
                continue;
            }
            if let Some(cwd) = Self::extract_cwd_from_file(&path) {
                return Some(cwd);
            }
        }
        None
    }

    fn list_project_paths(&self) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut out = Vec::new();
        for cwd in self.scan_all().into_values() {
            if seen.insert(cwd.clone()) {
                out.push(cwd);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_session_id_from_real_filename() {
        let f = "rollout-2026-05-20T10-22-16-019e4330-dd74-7a90-9eb0-158a7bad2183.jsonl";
        assert_eq!(
            CodexScanner::extract_session_id_from_filename(f).as_deref(),
            Some("019e4330-dd74-7a90-9eb0-158a7bad2183")
        );
    }

    #[test]
    fn extract_session_id_rejects_malformed() {
        assert_eq!(
            CodexScanner::extract_session_id_from_filename("rollout-broken.jsonl"),
            None
        );
        assert_eq!(
            CodexScanner::extract_session_id_from_filename("other.jsonl"),
            None
        );
    }

    #[test]
    fn scan_real_codex_dir_if_exists() {
        let scanner = CodexScanner;
        let map = scanner.scan_all();
        // 不强断言数量（CI 上可能没有 ~/.codex），只确认不 panic
        println!("scanned {} codex sessions", map.len());
    }
}
