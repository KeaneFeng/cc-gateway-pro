#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::State;

use crate::store::AppState;

/// 项目路由信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRoutingInfo {
    /// 项目路径
    pub project_path: String,
    /// 绑定的 Provider ID（如果有）
    pub provider_id: Option<String>,
    /// 绑定的 Provider 名称（如果有）
    pub provider_name: Option<String>,
    /// 该项目关联的会话数量
    pub session_count: usize,
}

/// 项目路由概览响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRoutingOverview {
    /// 所有已发现的项目路由信息
    pub projects: Vec<ProjectRoutingInfo>,
    /// 所有可用的 Provider 列表（claude 应用）
    pub available_providers: Vec<ProviderOption>,
}

/// Provider 选项（用于前端下拉选择）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderOption {
    pub id: String,
    pub name: String,
}

/// 从 ~/.claude/projects/ 目录扫描所有项目路径
/// 目录结构: ~/.claude/projects/<encoded-path>/<session-id>.jsonl
fn scan_project_dirs() -> Vec<String> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let projects_dir = home.join(".claude").join("projects");
    if !projects_dir.exists() {
        return Vec::new();
    }

    let mut project_paths: Vec<String> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&projects_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            // 先进入子目录（如 -Users-keane-www-cc-gateway/）
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
                                    if let Some(cwd) = json.get("cwd").and_then(|v| v.as_str()) {
                                        if !project_paths.contains(&cwd.to_string()) {
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

/// 从 ~/.claude/projects/ 目录扫描所有会话到项目的映射
/// 目录结构: ~/.claude/projects/<encoded-path>/<session-id>.jsonl
fn scan_session_projects() -> HashMap<String, String> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let projects_dir = home.join(".claude").join("projects");
    if !projects_dir.exists() {
        return HashMap::new();
    }

    let mut session_projects: HashMap<String, String> = HashMap::new();

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
                                        session_projects
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

    session_projects
}

/// 获取项目路由概览：所有已发现项目 + 当前绑定的 Provider + 可用 Provider 列表
#[tauri::command]
pub async fn get_project_routing(
    state: State<'_, AppState>,
) -> Result<ProjectRoutingOverview, String> {
    let db = state.db.clone();

    tauri::async_runtime::spawn_blocking(move || {
        // 扫描项目目录
        let project_paths = scan_project_dirs();
        let session_projects = scan_session_projects();

        // 从数据库读取 provider meta 中的 project_providers 映射
        let providers = db
            .get_all_providers("claude")
            .map_err(|e| format!("获取供应商列表失败: {e}"))?;

        // 构建 provider_id -> provider_name 映射
        let mut provider_names: HashMap<String, String> = HashMap::new();
        let mut available_providers: Vec<ProviderOption> = Vec::new();
        for (id, provider) in &providers {
            provider_names.insert(id.clone(), provider.name.clone());
            available_providers.push(ProviderOption {
                id: id.clone(),
                name: provider.name.clone(),
            });
        }

        // 从数据库设置中获取 project_providers 映射
        let project_providers: HashMap<String, String> = match db.get_setting("project_providers") {
            Ok(Some(json_str)) => {
                serde_json::from_str(&json_str).unwrap_or_default()
            }
            _ => HashMap::new(),
        };

        // 统计每个项目的会话数量
        let mut project_session_counts: HashMap<String, usize> = HashMap::new();
        for (_, project_path) in &session_projects {
            *project_session_counts
                .entry(project_path.clone())
                .or_insert(0) += 1;
        }

        // 构建项目路由信息列表
        let mut projects: Vec<ProjectRoutingInfo> = Vec::new();
        for project_path in &project_paths {
            let provider_id = project_providers.get(project_path).cloned();
            let provider_name = provider_id
                .as_ref()
                .and_then(|pid| provider_names.get(pid).cloned());
            let session_count = project_session_counts.get(project_path).copied().unwrap_or(0);

            projects.push(ProjectRoutingInfo {
                project_path: project_path.clone(),
                provider_id,
                provider_name,
                session_count,
            });
        }

        // 按项目路径排序
        projects.sort_by(|a, b| a.project_path.cmp(&b.project_path));

        Ok(ProjectRoutingOverview {
            projects,
            available_providers,
        })
    })
    .await
    .map_err(|e| format!("获取项目路由失败: {e}"))?
}

/// 设置某个项目路径绑定到某个 provider_id
#[tauri::command]
pub async fn set_project_provider(
    project_path: String,
    provider_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.clone();

    tauri::async_runtime::spawn_blocking(move || {
        // 读取现有的 project_providers 映射
        let mut project_providers: HashMap<String, String> =
            match db.get_setting("project_providers") {
                Ok(Some(json_str)) => serde_json::from_str(&json_str).unwrap_or_default(),
                _ => HashMap::new(),
            };

        project_providers.insert(project_path, provider_id);

        let json_str = serde_json::to_string(&project_providers)
            .map_err(|e| format!("序列化 project_providers 失败: {e}"))?;

        db.set_setting("project_providers", &json_str)
            .map_err(|e| format!("保存 project_providers 失败: {e}"))?;

        Ok(())
    })
    .await
    .map_err(|e| format!("设置项目供应商失败: {e}"))?
}

/// 移除某个项目的 provider 绑定
#[tauri::command]
pub async fn remove_project_provider(
    project_path: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let mut project_providers: HashMap<String, String> =
            match db.get_setting("project_providers") {
                Ok(Some(json_str)) => serde_json::from_str(&json_str).unwrap_or_default(),
                _ => HashMap::new(),
            };

        project_providers.remove(&project_path);

        let json_str = serde_json::to_string(&project_providers)
            .map_err(|e| format!("序列化 project_providers 失败: {e}"))?;

        db.set_setting("project_providers", &json_str)
            .map_err(|e| format!("保存 project_providers 失败: {e}"))?;

        Ok(())
    })
    .await
    .map_err(|e| format!("移除项目供应商失败: {e}"))?
}

/// 重新扫描 ~/.claude/projects/ 目录，刷新会话到项目的映射
#[tauri::command]
pub async fn refresh_session_projects(
    state: State<'_, AppState>,
) -> Result<ProjectRoutingOverview, String> {
    // refresh 就是重新调用 get_project_routing，因为扫描是实时进行的
    get_project_routing(state).await
}
