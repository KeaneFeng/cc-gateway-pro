#![allow(non_snake_case)]

use serde_json::{json, Value};
use std::path::PathBuf;
use tauri::State;
use tauri_plugin_dialog::DialogExt;

use crate::commands::sync_support::{
    post_sync_warning_from_result, run_post_import_sync, success_payload_with_warning,
};
use crate::database::backup::BackupEntry;
use crate::database::Database;
use crate::error::AppError;
use crate::provider::Provider;
use crate::services::provider::ProviderService;
use crate::store::AppState;

// ─── File import/export ──────────────────────────────────────

/// 导出数据库为 SQL 备份
#[tauri::command]
pub async fn export_config_to_file(
    #[allow(non_snake_case)] filePath: String,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let db = state.db.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let target_path = PathBuf::from(&filePath);
        db.export_sql(&target_path)?;
        Ok::<_, AppError>(json!({
            "success": true,
            "message": "SQL exported successfully",
            "filePath": filePath
        }))
    })
    .await
    .map_err(|e| format!("导出配置失败: {e}"))?
    .map_err(|e: AppError| e.to_string())
}

/// 从 SQL 备份导入数据库
#[tauri::command]
pub async fn import_config_from_file(
    #[allow(non_snake_case)] filePath: String,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let db = state.db.clone();
    let db_for_sync = db.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let path_buf = PathBuf::from(&filePath);
        let backup_id = db.import_sql(&path_buf)?;
        let warning = post_sync_warning_from_result(Ok(run_post_import_sync(db_for_sync)));
        if let Some(msg) = warning.as_ref() {
            log::warn!("[Import] post-import sync warning: {msg}");
        }
        Ok::<_, AppError>(success_payload_with_warning(backup_id, warning))
    })
    .await
    .map_err(|e| format!("导入配置失败: {e}"))?
    .map_err(|e: AppError| e.to_string())
}

#[tauri::command]
pub async fn sync_current_providers_live(state: State<'_, AppState>) -> Result<Value, String> {
    let db = state.db.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let app_state = AppState::new(db);
        ProviderService::sync_current_to_live(&app_state)?;
        Ok::<_, AppError>(json!({
            "success": true,
            "message": "Live configuration synchronized"
        }))
    })
    .await
    .map_err(|e| format!("同步当前供应商失败: {e}"))?
    .map_err(|e: AppError| e.to_string())
}

// ─── File dialogs ────────────────────────────────────────────

/// 保存文件对话框
#[tauri::command]
pub async fn save_file_dialog<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    #[allow(non_snake_case)] defaultName: String,
) -> Result<Option<String>, String> {
    let dialog = app.dialog();
    let result = dialog
        .file()
        .add_filter("SQL", &["sql"])
        .set_file_name(&defaultName)
        .blocking_save_file();

    Ok(result.map(|p| p.to_string()))
}

/// 打开文件对话框
#[tauri::command]
pub async fn open_file_dialog<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Option<String>, String> {
    let dialog = app.dialog();
    let result = dialog
        .file()
        .add_filter("SQL", &["sql"])
        .blocking_pick_file();

    Ok(result.map(|p| p.to_string()))
}

/// 打开 ZIP 文件选择对话框
#[tauri::command]
pub async fn open_zip_file_dialog<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Option<String>, String> {
    let dialog = app.dialog();
    let result = dialog
        .file()
        .add_filter("ZIP / Skill", &["zip", "skill"])
        .blocking_pick_file();

    Ok(result.map(|p| p.to_string()))
}

// ─── Database backup management ─────────────────────────────

/// Manually create a database backup
#[tauri::command]
pub async fn create_db_backup(state: State<'_, AppState>) -> Result<String, String> {
    let db = state.db.clone();
    tauri::async_runtime::spawn_blocking(move || match db.backup_database_file()? {
        Some(path) => Ok(path
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_default()),
        None => Err(AppError::Config(
            "Database file not found, backup skipped".to_string(),
        )),
    })
    .await
    .map_err(|e| format!("Backup failed: {e}"))?
    .map_err(|e: AppError| e.to_string())
}

/// List all database backup files
#[tauri::command]
pub fn list_db_backups() -> Result<Vec<BackupEntry>, String> {
    Database::list_backups().map_err(|e| e.to_string())
}

/// Restore database from a backup file
#[tauri::command]
pub async fn restore_db_backup(
    state: State<'_, AppState>,
    filename: String,
) -> Result<String, String> {
    let db = state.db.clone();
    tauri::async_runtime::spawn_blocking(move || db.restore_from_backup(&filename))
        .await
        .map_err(|e| format!("Restore failed: {e}"))?
        .map_err(|e: AppError| e.to_string())
}

/// Rename a database backup file
#[tauri::command]
pub fn rename_db_backup(
    #[allow(non_snake_case)] oldFilename: String,
    #[allow(non_snake_case)] newName: String,
) -> Result<String, String> {
    Database::rename_backup(&oldFilename, &newName).map_err(|e| e.to_string())
}

/// Delete a database backup file
#[tauri::command]
pub fn delete_db_backup(filename: String) -> Result<(), String> {
    Database::delete_backup(&filename).map_err(|e| e.to_string())
}

/// 从 cc-switch 同步供应商到 cc-gateway-pro
#[tauri::command]
pub async fn sync_from_cc_switch(state: State<'_, AppState>) -> Result<Value, String> {
    let db = state.db.clone();
    tauri::async_runtime::spawn_blocking(move || {
        // 获取 cc-switch 数据库路径（旧版应用）
        let home = dirs::home_dir().ok_or_else(|| AppError::Config("无法获取用户主目录".to_string()))?;
        let legacy_db_path = home.join(".cc-switch").join("cc-switch.db");

        if !legacy_db_path.exists() {
            return Err(AppError::Config(format!(
                "cc-switch 数据库不存在: {}",
                legacy_db_path.display()
            )));
        }

        // 打开 cc-switch 数据库（只读）
        let src_conn = rusqlite::Connection::open_with_flags(
            &legacy_db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .map_err(|e| AppError::Database(format!("无法打开 cc-switch 数据库: {e}")))?;

        // 查询 cc-switch 中所有供应商（涵盖 claude/codex/gemini/opencode/hermes 等）
        // 注：claude-desktop 不通过该路径同步（其配置走独立的 3P profile 流程）
        let mut stmt = src_conn
            .prepare(
                "SELECT id, app_type, name, settings_config, website_url, category, created_at, sort_index, notes, icon, icon_color, meta
                 FROM providers
                 WHERE app_type IN ('claude', 'codex', 'gemini', 'opencode', 'hermes')"
            )
            .map_err(|e| AppError::Database(format!("查询 cc-switch 供应商失败: {e}")))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,          // id
                    row.get::<_, String>(1)?,          // app_type
                    row.get::<_, String>(2)?,          // name
                    row.get::<_, String>(3)?,          // settings_config
                    row.get::<_, Option<String>>(4)?,  // website_url
                    row.get::<_, Option<String>>(5)?,  // category
                    row.get::<_, Option<i64>>(6)?,     // created_at
                    row.get::<_, Option<usize>>(7)?,   // sort_index
                    row.get::<_, Option<String>>(8)?,  // notes
                    row.get::<_, Option<String>>(9)?,  // icon
                    row.get::<_, Option<String>>(10)?, // icon_color
                    row.get::<_, Option<String>>(11)?, // meta
                ))
            })
            .map_err(|e| AppError::Database(format!("读取 cc-switch 供应商失败: {e}")))?;

        let mut synced_count = 0usize;
        let mut error_count = 0usize;
        let mut per_app: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();

        for row in rows {
            match row {
                Ok((id, app_type, name, settings_config_str, website_url, category, created_at, sort_index, notes, icon, icon_color, meta_str)) => {
                    let settings_config: Value = serde_json::from_str(&settings_config_str)
                        .unwrap_or(Value::Null);

                    // 解析 meta，若 cc-switch 有 meta 字段则使用，否则用默认值
                    let mut meta = meta_str
                        .as_deref()
                        .and_then(|s| serde_json::from_str::<crate::provider::ProviderMeta>(s).ok())
                        .unwrap_or_default();

                    // 清空 custom_endpoints，避免跨应用数据污染
                    meta.custom_endpoints.clear();

                    // category：尊重源数据原值；仅当 claude 且为空时回填默认 "pro"
                    // （为保持 fork 历史行为；其他 app_type 不应被强制改写）
                    let category = match (app_type.as_str(), category) {
                        ("claude", None) => Some("pro".to_string()),
                        (_, c) => c,
                    };

                    let provider = Provider {
                        id: id.clone(),
                        name,
                        settings_config,
                        website_url,
                        category,
                        created_at,
                        sort_index,
                        notes,
                        meta: Some(meta),
                        icon,
                        icon_color,
                        in_failover_queue: false,
                    };

                    match db.save_provider(&app_type, &provider) {
                        Ok(()) => {
                            synced_count += 1;
                            *per_app.entry(app_type).or_insert(0) += 1;
                        }
                        Err(e) => {
                            log::warn!("[SyncFromCcSwitch] 保存供应商 {app_type}/{id} 失败: {e}");
                            error_count += 1;
                        }
                    }
                }
                Err(e) => {
                    log::warn!("[SyncFromCcSwitch] 读取行失败: {e}");
                    error_count += 1;
                }
            }
        }

        let detail = per_app
            .iter()
            .map(|(app, n)| format!("{app}={n}"))
            .collect::<Vec<_>>()
            .join(", ");
        log::info!(
            "[SyncFromCcSwitch] 同步完成: 成功 {synced_count} 个 ({detail}), 失败 {error_count} 个"
        );
        let by_app: serde_json::Map<String, Value> = per_app
            .into_iter()
            .map(|(k, v)| (k, Value::from(v)))
            .collect();
        Ok::<_, AppError>(json!({
            "success": true,
            "syncedCount": synced_count,
            "errorCount": error_count,
            "byApp": by_app,
            "message": if detail.is_empty() {
                format!("同步完成: {} 个供应商已导入", synced_count)
            } else {
                format!("同步完成: {} 个供应商已导入 ({})", synced_count, detail)
            }
        }))
    })
    .await
    .map_err(|e| format!("同步 cc-switch 供应商失败: {e}"))?
    .map_err(|e: AppError| e.to_string())
}
