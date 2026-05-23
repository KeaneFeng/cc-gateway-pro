#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::State;

use crate::store::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRoutingInfo {
    pub project_path: String,
    pub provider_id: Option<String>,
    pub provider_name: Option<String>,
    pub provider_notes: Option<String>,
    pub session_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRoutingOverview {
    pub projects: Vec<ProjectRoutingInfo>,
    pub available_providers: Vec<ProviderOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderOption {
    pub id: String,
    pub name: String,
    pub notes: Option<String>,
}

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
                                        // 过滤掉 .claude 目录下的路径（如 .claude/skills）
                                        if !cwd.is_empty()
                                            && !cwd.contains("/.claude/")
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
                                        if !cwd.is_empty() {
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
    }
    session_projects
}

#[tauri::command]
pub async fn get_project_routing(
    state: State<'_, AppState>,
) -> Result<ProjectRoutingOverview, String> {
    get_project_routing_for_app("claude".to_string(), state).await
}

#[tauri::command]
pub async fn set_project_provider(
    project_path: String,
    provider_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    set_project_provider_for_app("claude".to_string(), project_path, provider_id, state).await
}

#[tauri::command]
pub async fn remove_project_provider(
    project_path: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    remove_project_provider_for_app("claude".to_string(), project_path, state).await
}

#[tauri::command]
pub async fn refresh_session_projects(
    state: State<'_, AppState>,
) -> Result<ProjectRoutingOverview, String> {
    get_project_routing(state).await
}

#[tauri::command]
pub async fn get_project_routing_for_app(
    app: String,
    state: State<'_, AppState>,
) -> Result<ProjectRoutingOverview, String> {
    let db = state.db.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let project_paths = match app.as_str() {
            "codex" => {
                use crate::proxy::project_router::codex::CodexScanner;
                use crate::proxy::project_router::scanner::SessionProjectScanner;
                CodexScanner.list_project_paths()
            }
            _ => scan_project_dirs(),
        };
        let session_projects = match app.as_str() {
            "codex" => {
                use crate::proxy::project_router::codex::CodexScanner;
                use crate::proxy::project_router::scanner::SessionProjectScanner;
                CodexScanner.scan_all()
            }
            _ => scan_session_projects(),
        };
        let providers = db
            .get_all_providers(&app)
            .map_err(|e| format!("Get providers failed: {e}"))?;
        let mut provider_names: HashMap<String, String> = HashMap::new();
        let mut provider_notes_map: HashMap<String, String> = HashMap::new();
        let mut available_providers: Vec<ProviderOption> = Vec::new();
        for (id, provider) in &providers {
            provider_names.insert(id.clone(), provider.name.clone());
            if let Some(ref notes) = provider.notes {
                provider_notes_map.insert(id.clone(), notes.clone());
            }
            available_providers.push(ProviderOption {
                id: id.clone(),
                name: provider.name.clone(),
                notes: provider.notes.clone(),
            });
        }
        let settings_key = match app.as_str() {
            "codex" => "project_providers_codex",
            _ => "project_providers",
        };
        let project_providers: HashMap<String, String> = match db.get_setting(settings_key) {
            Ok(Some(json_str)) => serde_json::from_str(&json_str).unwrap_or_default(),
            _ => HashMap::new(),
        };
        let mut project_session_counts: HashMap<String, usize> = HashMap::new();
        for project_path in session_projects.values() {
            *project_session_counts
                .entry(project_path.clone())
                .or_insert(0) += 1;
        }
        let mut projects: Vec<ProjectRoutingInfo> = Vec::new();
        for project_path in &project_paths {
            let provider_id = project_providers.get(project_path).cloned();
            let provider_name = provider_id
                .as_ref()
                .and_then(|pid| provider_names.get(pid).cloned());
            let session_count = project_session_counts
                .get(project_path)
                .copied()
                .unwrap_or(0);
            let provider_notes = provider_id
                .as_ref()
                .and_then(|pid| provider_notes_map.get(pid).cloned());
            projects.push(ProjectRoutingInfo {
                project_path: project_path.clone(),
                provider_id,
                provider_name,
                provider_notes,
                session_count,
            });
        }
        projects.sort_by(|a, b| a.project_path.cmp(&b.project_path));
        Ok(ProjectRoutingOverview {
            projects,
            available_providers,
        })
    })
    .await
    .map_err(|e| format!("Get project routing failed: {e}"))?
}

#[tauri::command]
pub async fn set_project_provider_for_app(
    app: String,
    project_path: String,
    provider_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let settings_key = match app.as_str() {
            "codex" => "project_providers_codex",
            _ => "project_providers",
        };
        let mut project_providers: HashMap<String, String> = match db.get_setting(settings_key) {
            Ok(Some(json_str)) => serde_json::from_str(&json_str).unwrap_or_default(),
            _ => HashMap::new(),
        };
        project_providers.insert(project_path, provider_id);
        let json_str = serde_json::to_string(&project_providers)
            .map_err(|e| format!("Serialize failed: {e}"))?;
        db.set_setting(settings_key, &json_str)
            .map_err(|e| format!("Save failed: {e}"))?;
        Ok(())
    })
    .await
    .map_err(|e| format!("Set project provider failed: {e}"))?
}

#[tauri::command]
pub async fn remove_project_provider_for_app(
    app: String,
    project_path: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let settings_key = match app.as_str() {
            "codex" => "project_providers_codex",
            _ => "project_providers",
        };
        let mut project_providers: HashMap<String, String> = match db.get_setting(settings_key) {
            Ok(Some(json_str)) => serde_json::from_str(&json_str).unwrap_or_default(),
            _ => HashMap::new(),
        };
        project_providers.remove(&project_path);
        let json_str = serde_json::to_string(&project_providers)
            .map_err(|e| format!("Serialize failed: {e}"))?;
        db.set_setting(settings_key, &json_str)
            .map_err(|e| format!("Save failed: {e}"))?;
        Ok(())
    })
    .await
    .map_err(|e| format!("Remove project provider failed: {e}"))?
}
