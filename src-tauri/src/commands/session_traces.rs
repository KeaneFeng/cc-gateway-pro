use crate::services::sql_helpers::fresh_input_sql;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const LOCAL_CONTEXT_SCAN_MAX_ENTRIES: usize = 20_000;
const LOCAL_CONTEXT_SCAN_MAX_DURATION: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceSessionFilters {
    pub app_type: Option<String>,
    pub search: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceSessionSummary {
    pub session_id: String,
    pub title: Option<String>,
    pub app_type: String,
    pub provider_id: Option<String>,
    pub model: Option<String>,
    pub turn_count: u32,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_read_tokens: u64,
    pub total_cache_creation_tokens: u64,
    pub total_cost_usd: String,
    pub avg_latency_ms: Option<u64>,
    pub last_status_code: Option<u16>,
    pub first_seen_at: i64,
    pub last_seen_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionTraceSettingsPayload {
    pub enabled: bool,
    pub mode: String,
    pub retention_days: u32,
    pub max_response_text_chars: u32,
    pub capture_request_json: bool,
    pub capture_response_json: bool,
    pub redact_sensitive_values: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceSessionDetailRequest {
    pub session_id: String,
    pub app_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceTurnDetail {
    pub trace_id: String,
    pub turn_index: Option<u32>,
    pub provider_id: Option<String>,
    pub model: Option<String>,
    pub request_model: Option<String>,
    pub is_streaming: bool,
    pub status_code: Option<u16>,
    pub system_prompt_preview: Option<String>,
    pub system_prompt_hash: Option<String>,
    pub message_count: u32,
    pub tool_count: u32,
    pub request_summary: Value,
    pub context_stats: Value,
    pub context_window_tokens: Option<u64>,
    pub context_used_tokens: Option<u64>,
    pub context_usage_ratio: Option<f64>,
    pub response_text_preview: Option<String>,
    pub tool_calls: Value,
    pub stop_reason: Option<String>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
    pub latency_ms: Option<u64>,
    pub first_token_ms: Option<u64>,
    pub trace_mode: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceSessionDetail {
    pub session_id: String,
    pub title: Option<String>,
    pub app_type: Option<String>,
    pub turn_count: u32,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_read_tokens: u64,
    pub total_cache_creation_tokens: u64,
    pub avg_context_usage_ratio: Option<f64>,
    pub max_context_used_tokens: Option<u64>,
    pub traces: Vec<TraceTurnDetail>,
}

fn settings_to_payload(
    settings: crate::settings::SessionTraceSettings,
) -> SessionTraceSettingsPayload {
    SessionTraceSettingsPayload {
        enabled: settings.enabled,
        mode: match settings.mode {
            crate::settings::SessionTraceMode::Off => "off",
            crate::settings::SessionTraceMode::Summary => "summary",
            crate::settings::SessionTraceMode::Full => "full",
        }
        .to_string(),
        retention_days: settings.retention_days,
        max_response_text_chars: settings.max_response_text_chars,
        capture_request_json: settings.capture_request_json,
        capture_response_json: settings.capture_response_json,
        redact_sensitive_values: settings.redact_sensitive_values,
    }
}

fn payload_to_settings(
    payload: SessionTraceSettingsPayload,
) -> crate::settings::SessionTraceSettings {
    let mode = match payload.mode.as_str() {
        "summary" => crate::settings::SessionTraceMode::Summary,
        "full" => crate::settings::SessionTraceMode::Full,
        _ => crate::settings::SessionTraceMode::Off,
    };

    let mut settings = crate::settings::SessionTraceSettings {
        enabled: payload.enabled,
        mode,
        retention_days: payload.retention_days,
        max_response_text_chars: payload.max_response_text_chars,
        capture_request_json: payload.capture_request_json,
        capture_response_json: payload.capture_response_json,
        redact_sensitive_values: payload.redact_sensitive_values,
    };
    settings.normalize();
    settings
}

#[tauri::command]
pub async fn get_session_trace_settings() -> Result<SessionTraceSettingsPayload, String> {
    Ok(settings_to_payload(
        crate::settings::get_settings().session_traces,
    ))
}

#[tauri::command]
pub async fn set_session_trace_settings(
    settings: SessionTraceSettingsPayload,
) -> Result<SessionTraceSettingsPayload, String> {
    let next = payload_to_settings(settings);
    let payload = settings_to_payload(next.clone());
    let mut app_settings = crate::settings::get_settings();
    app_settings.session_traces = next;
    crate::settings::update_settings(app_settings).map_err(|e| e.to_string())?;
    Ok(payload)
}

#[tauri::command]
pub async fn list_trace_sessions(
    state: tauri::State<'_, crate::AppState>,
    filters: Option<TraceSessionFilters>,
) -> Result<Vec<TraceSessionSummary>, String> {
    let filters = filters.unwrap_or(TraceSessionFilters {
        app_type: None,
        search: None,
        limit: None,
    });
    let limit = filters.limit.unwrap_or(200).clamp(1, 500);
    let search = filters
        .search
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{}%", s.replace('%', "\\%").replace('_', "\\_")));

    let conn = state
        .db
        .conn
        .lock()
        .map_err(|e| format!("Mutex lock failed: {e}"))?;
    let fresh_input = fresh_input_sql("l");
    let mut conditions = vec![
        "session_id IS NOT NULL".to_string(),
        "TRIM(session_id) <> ''".to_string(),
    ];
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(app_type) = filters
        .app_type
        .filter(|v| !v.trim().is_empty() && v != "all")
    {
        conditions.push("app_type = ?".to_string());
        params.push(Box::new(app_type));
    }

    if let Some(search) = search {
        conditions.push("(session_id LIKE ? ESCAPE '\\' OR model LIKE ? ESCAPE '\\' OR request_model LIKE ? ESCAPE '\\' OR request_summary_json LIKE ? ESCAPE '\\')".to_string());
        params.push(Box::new(search.clone()));
        params.push(Box::new(search.clone()));
        params.push(Box::new(search.clone()));
        params.push(Box::new(search));
    }

    params.push(Box::new(limit as i64));
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let where_clause = conditions.join(" AND ");
    let sql = format!(
        "WITH trace_rows AS (
            SELECT
                session_id,
                app_type,
                provider_id,
                model,
                request_model,
                input_tokens,
                output_tokens,
                cache_read_tokens,
                cache_creation_tokens,
                0.0 AS total_cost,
                latency_ms,
                status_code,
                created_at,
                request_summary_json,
                request_json,
                response_text_preview,
                1 AS trace_count,
                0 AS log_count
            FROM session_traces
        ),
        log_rows AS (
            SELECT
                l.session_id,
                l.app_type,
                l.provider_id,
                l.model,
                l.request_model,
                {fresh_input} AS input_tokens,
                l.output_tokens,
                l.cache_read_tokens,
                l.cache_creation_tokens,
                CAST(l.total_cost_usd AS REAL) AS total_cost,
                l.latency_ms,
                l.status_code,
                l.created_at,
                '{{}}' AS request_summary_json,
                NULL AS request_json,
                NULL AS response_text_preview,
                0 AS trace_count,
                1 AS log_count
            FROM proxy_request_logs l
        ),
        source_rows AS (
            SELECT * FROM trace_rows
            UNION ALL
            SELECT * FROM log_rows
        ),
        filtered AS (
            SELECT *
            FROM source_rows
            WHERE {where_clause}
        ),
        grouped AS (
            SELECT
                session_id,
                app_type,
                SUM(trace_count) AS trace_count,
                SUM(log_count) AS log_count,
                COALESCE(SUM(CASE WHEN trace_count = 1 THEN input_tokens ELSE 0 END), 0) AS trace_input_tokens,
                COALESCE(SUM(CASE WHEN trace_count = 1 THEN output_tokens ELSE 0 END), 0) AS trace_output_tokens,
                COALESCE(SUM(CASE WHEN trace_count = 1 THEN cache_read_tokens ELSE 0 END), 0) AS trace_cache_read_tokens,
                COALESCE(SUM(CASE WHEN trace_count = 1 THEN cache_creation_tokens ELSE 0 END), 0) AS trace_cache_creation_tokens,
                AVG(CASE WHEN trace_count = 1 THEN NULLIF(latency_ms, 0) END) AS trace_avg_latency_ms,
                COALESCE(SUM(CASE WHEN log_count = 1 THEN input_tokens ELSE 0 END), 0) AS log_input_tokens,
                COALESCE(SUM(CASE WHEN log_count = 1 THEN output_tokens ELSE 0 END), 0) AS log_output_tokens,
                COALESCE(SUM(CASE WHEN log_count = 1 THEN cache_read_tokens ELSE 0 END), 0) AS log_cache_read_tokens,
                COALESCE(SUM(CASE WHEN log_count = 1 THEN cache_creation_tokens ELSE 0 END), 0) AS log_cache_creation_tokens,
                COALESCE(SUM(CASE WHEN log_count = 1 THEN total_cost ELSE 0 END), 0) AS log_total_cost,
                AVG(CASE WHEN log_count = 1 THEN NULLIF(latency_ms, 0) END) AS log_avg_latency_ms,
                MIN(created_at) AS first_seen_at,
                MAX(created_at) AS last_seen_at
            FROM filtered
            GROUP BY session_id, app_type
        )
        SELECT
            g.session_id,
            g.app_type,
            (
                SELECT request_summary_json
                FROM filtered f
                WHERE f.session_id = g.session_id AND f.app_type = g.app_type AND f.trace_count = 1
                ORDER BY f.created_at ASC
                LIMIT 1
            ) AS title_summary_json,
            (
                SELECT request_json
                FROM filtered f
                WHERE f.session_id = g.session_id AND f.app_type = g.app_type AND f.trace_count = 1
                ORDER BY f.created_at ASC
                LIMIT 1
            ) AS title_request_json,
            (
                SELECT response_text_preview
                FROM filtered f
                WHERE f.session_id = g.session_id AND f.app_type = g.app_type AND f.trace_count = 1
                ORDER BY f.created_at ASC
                LIMIT 1
            ) AS title_response_preview,
            (
                SELECT provider_id
                FROM filtered f
                WHERE f.session_id = g.session_id AND f.app_type = g.app_type
                ORDER BY f.created_at DESC
                LIMIT 1
            ) AS provider_id,
            (
                SELECT model
                FROM filtered f
                WHERE f.session_id = g.session_id AND f.app_type = g.app_type
                ORDER BY f.created_at DESC
                LIMIT 1
            ) AS model,
            CASE WHEN g.trace_count > 0 THEN g.trace_count ELSE g.log_count END AS turn_count,
            CASE WHEN g.trace_count > 0 THEN g.trace_input_tokens ELSE g.log_input_tokens END AS input_tokens,
            CASE WHEN g.trace_count > 0 THEN g.trace_output_tokens ELSE g.log_output_tokens END AS output_tokens,
            CASE WHEN g.trace_count > 0 THEN g.trace_cache_read_tokens ELSE g.log_cache_read_tokens END AS cache_read_tokens,
            CASE WHEN g.trace_count > 0 THEN g.trace_cache_creation_tokens ELSE g.log_cache_creation_tokens END AS cache_creation_tokens,
            g.log_total_cost,
            CASE WHEN g.trace_count > 0 THEN g.trace_avg_latency_ms ELSE g.log_avg_latency_ms END AS avg_latency_ms,
            (
                SELECT status_code
                FROM filtered f
                WHERE f.session_id = g.session_id AND f.app_type = g.app_type
                ORDER BY f.created_at DESC
                LIMIT 1
            ) AS last_status_code,
            g.first_seen_at,
            g.last_seen_at
        FROM grouped g
        ORDER BY g.last_seen_at DESC
        LIMIT ?"
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(param_refs.as_slice(), |row| {
            let title_summary: Option<String> = row.get(2)?;
            let title_request: Option<String> = row.get(3)?;
            let title_response: Option<String> = row.get(4)?;
            let avg_latency: Option<f64> = row.get(13)?;
            let status_code: Option<i64> = row.get(14)?;
            Ok(TraceSessionSummary {
                session_id: row.get(0)?,
                title: title_from_trace_sources(
                    title_summary.as_deref(),
                    title_request.as_deref(),
                    title_response.as_deref(),
                ),
                app_type: row.get(1)?,
                provider_id: row.get(5)?,
                model: row.get(6)?,
                turn_count: row.get::<_, i64>(7)? as u32,
                total_input_tokens: row.get::<_, i64>(8)? as u64,
                total_output_tokens: row.get::<_, i64>(9)? as u64,
                total_cache_read_tokens: row.get::<_, i64>(10)? as u64,
                total_cache_creation_tokens: row.get::<_, i64>(11)? as u64,
                total_cost_usd: format!("{:.6}", row.get::<_, f64>(12)?),
                avg_latency_ms: avg_latency.map(|v| v.round() as u64),
                last_status_code: status_code.map(|v| v as u16),
                first_seen_at: row.get(15)?,
                last_seen_at: row.get(16)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut sessions = Vec::new();
    for row in rows {
        sessions.push(row.map_err(|e| e.to_string())?);
    }
    Ok(sessions)
}

#[tauri::command]
pub async fn get_trace_session_detail(
    state: tauri::State<'_, crate::AppState>,
    request: TraceSessionDetailRequest,
) -> Result<TraceSessionDetail, String> {
    let session_id = request.session_id.trim().to_string();
    if session_id.is_empty() {
        return Err("session_id is required".to_string());
    }

    let conn = state
        .db
        .conn
        .lock()
        .map_err(|e| format!("Mutex lock failed: {e}"))?;

    let mut conditions = vec!["session_id = ?".to_string()];
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(session_id.clone())];
    if let Some(app_type) = request
        .app_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "all")
    {
        conditions.push("app_type = ?".to_string());
        params.push(Box::new(app_type.to_string()));
    }

    let where_clause = conditions.join(" AND ");
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let sql = format!(
        "SELECT
            trace_id,
            turn_index,
            provider_id,
            model,
            request_model,
            is_streaming,
            status_code,
            system_prompt_preview,
            system_prompt_hash,
            message_count,
            tool_count,
            request_summary_json,
            context_stats_json,
            context_window_tokens,
            context_used_tokens,
            context_usage_ratio,
            response_text_preview,
            tool_calls_json,
            stop_reason,
            input_tokens,
            output_tokens,
            cache_read_tokens,
            cache_creation_tokens,
            latency_ms,
            first_token_ms,
            trace_mode,
            created_at,
            request_json
         FROM session_traces
         WHERE {where_clause}
         ORDER BY COALESCE(turn_index, 0) DESC, created_at DESC"
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(param_refs.as_slice(), |row| {
            let status_code: Option<i64> = row.get(6)?;
            let turn_index: Option<i64> = row.get(1)?;
            let request_json: Option<String> = row.get(27)?;
            let context_stats = parse_json_or(row.get::<_, String>(12)?, json!({}));
            let context_stats = crate::proxy::session_trace::enrich_context_stats_from_request(
                context_stats,
                request_json.as_deref(),
            );
            Ok(TraceTurnDetail {
                trace_id: row.get(0)?,
                turn_index: turn_index.map(|value| value as u32),
                provider_id: row.get(2)?,
                model: row.get(3)?,
                request_model: row.get(4)?,
                is_streaming: row.get::<_, i64>(5)? != 0,
                status_code: status_code.map(|value| value as u16),
                system_prompt_preview: row.get(7)?,
                system_prompt_hash: row.get(8)?,
                message_count: row.get::<_, i64>(9)? as u32,
                tool_count: row.get::<_, i64>(10)? as u32,
                request_summary: parse_json_or(row.get::<_, String>(11)?, json!({})),
                context_stats,
                context_window_tokens: row.get::<_, Option<i64>>(13)?.map(|value| value as u64),
                context_used_tokens: row.get::<_, Option<i64>>(14)?.map(|value| value as u64),
                context_usage_ratio: row.get(15)?,
                response_text_preview: row.get(16)?,
                tool_calls: parse_json_or(row.get::<_, String>(17)?, json!([])),
                stop_reason: row.get(18)?,
                input_tokens: row.get::<_, i64>(19)? as u64,
                output_tokens: row.get::<_, i64>(20)? as u64,
                cache_read_tokens: row.get::<_, i64>(21)? as u64,
                cache_creation_tokens: row.get::<_, i64>(22)? as u64,
                latency_ms: row.get::<_, Option<i64>>(23)?.map(|value| value as u64),
                first_token_ms: row.get::<_, Option<i64>>(24)?.map(|value| value as u64),
                trace_mode: row.get(25)?,
                created_at: row.get(26)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut traces = Vec::new();
    for row in rows {
        traces.push(row.map_err(|e| e.to_string())?);
    }
    let turn_count = traces.len() as u32;
    let total_input_tokens = traces.iter().map(|trace| trace.input_tokens).sum();
    let total_output_tokens = traces.iter().map(|trace| trace.output_tokens).sum();
    let total_cache_read_tokens = traces.iter().map(|trace| trace.cache_read_tokens).sum();
    let total_cache_creation_tokens = traces.iter().map(|trace| trace.cache_creation_tokens).sum();
    let context_ratios: Vec<f64> = traces
        .iter()
        .filter_map(|trace| trace.context_usage_ratio)
        .collect();
    let avg_context_usage_ratio = if context_ratios.is_empty() {
        None
    } else {
        Some(context_ratios.iter().sum::<f64>() / context_ratios.len() as f64)
    };
    let max_context_used_tokens = traces
        .iter()
        .filter_map(|trace| trace.context_used_tokens)
        .max();
    let title =
        resolve_trace_session_title(&conn, &where_clause, param_refs.as_slice()).or_else(|| {
            traces
                .iter()
                .rev()
                .find_map(|trace| title_from_request_summary_value(&trace.request_summary))
        });
    drop(stmt);
    drop(conn);

    if should_load_local_context_usage_text(request.app_type.as_deref(), &traces) {
        let context_text = load_local_context_usage_text(&session_id, request.app_type.as_deref());
        if apply_local_context_enrichment_to_traces(&mut traces, context_text.as_deref()) {
            persist_trace_context_stats(&state, &traces);
        }
    }

    Ok(TraceSessionDetail {
        session_id,
        title,
        app_type: request.app_type,
        turn_count,
        total_input_tokens,
        total_output_tokens,
        total_cache_read_tokens,
        total_cache_creation_tokens,
        avg_context_usage_ratio,
        max_context_used_tokens,
        traces,
    })
}

fn parse_json_or(value: String, fallback: Value) -> Value {
    serde_json::from_str(&value).unwrap_or(fallback)
}

fn load_local_context_usage_text(session_id: &str, app_type: Option<&str>) -> Option<String> {
    if !matches!(app_type, None | Some("claude") | Some("claude-desktop")) {
        return None;
    }
    let path = find_claude_session_file(session_id)?;
    let content = std::fs::read_to_string(path).ok()?;
    let mut latest = None;
    for line in content.lines() {
        if !line.contains("Context Usage") {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let Some(text) = value
            .get("content")
            .and_then(Value::as_str)
            .or_else(|| value.pointer("/message/content").and_then(Value::as_str))
        else {
            continue;
        };
        if text.contains("Context Usage") {
            latest = Some(strip_ansi(text));
        }
    }
    latest
}

const LOCAL_CONTEXT_ENRICHMENT_KEY: &str = "localContextEnrichment";

fn should_load_local_context_usage_text(
    app_type: Option<&str>,
    traces: &[TraceTurnDetail],
) -> bool {
    if traces.is_empty() || !matches!(app_type, None | Some("claude") | Some("claude-desktop")) {
        return false;
    }
    traces
        .iter()
        .any(|trace| !has_local_context_enrichment_marker(&trace.context_stats))
}

fn has_local_context_enrichment_marker(context_stats: &Value) -> bool {
    context_stats
        .get(LOCAL_CONTEXT_ENRICHMENT_KEY)
        .and_then(|value| value.get("status"))
        .and_then(Value::as_str)
        .is_some()
}

fn apply_local_context_enrichment_to_traces(
    traces: &mut [TraceTurnDetail],
    context_text: Option<&str>,
) -> bool {
    let mut changed = false;
    for trace in traces {
        if has_local_context_enrichment_marker(&trace.context_stats) {
            continue;
        }

        trace.context_stats = match context_text {
            Some(text) => {
                let enriched = crate::proxy::session_trace::enrich_context_stats_from_context_text(
                    trace.context_stats.clone(),
                    text,
                );
                mark_local_context_enrichment(enriched, "found")
            }
            None => mark_local_context_enrichment(trace.context_stats.clone(), "missing"),
        };
        changed = true;
    }
    changed
}

fn mark_local_context_enrichment(mut context_stats: Value, status: &str) -> Value {
    if let Some(map) = context_stats.as_object_mut() {
        map.insert(
            LOCAL_CONTEXT_ENRICHMENT_KEY.to_string(),
            json!({
                "status": status,
                "source": "claude-jsonl-context-usage",
            }),
        );
        context_stats
    } else {
        json!({
            LOCAL_CONTEXT_ENRICHMENT_KEY: {
                "status": status,
                "source": "claude-jsonl-context-usage",
            }
        })
    }
}

fn persist_trace_context_stats(state: &crate::AppState, traces: &[TraceTurnDetail]) {
    let Ok(conn) = state.db.conn.lock() else {
        log::warn!("[SessionTraces] 无法锁定数据库以持久化 context enrich 状态");
        return;
    };
    let now = chrono::Utc::now().timestamp();
    for trace in traces {
        let Ok(context_stats_json) = serde_json::to_string(&trace.context_stats) else {
            continue;
        };
        if let Err(err) = conn.execute(
            "UPDATE session_traces
             SET context_stats_json = ?1, updated_at = ?2
             WHERE trace_id = ?3",
            rusqlite::params![context_stats_json, now, trace.trace_id],
        ) {
            log::warn!(
                "[SessionTraces] 持久化 trace context enrich 状态失败 trace_id={}: {err}",
                trace.trace_id
            );
        }
    }
}

fn find_claude_session_file(session_id: &str) -> Option<PathBuf> {
    let root = crate::config::get_claude_config_dir().join("projects");
    let mut budget = FileScanBudget::new(
        LOCAL_CONTEXT_SCAN_MAX_ENTRIES,
        LOCAL_CONTEXT_SCAN_MAX_DURATION,
    );
    find_file_by_name(&root, &format!("{session_id}.jsonl"), 4, &mut budget)
}

struct FileScanBudget {
    started_at: Instant,
    max_entries: usize,
    max_duration: Duration,
    visited_entries: usize,
}

impl FileScanBudget {
    fn new(max_entries: usize, max_duration: Duration) -> Self {
        Self {
            started_at: Instant::now(),
            max_entries,
            max_duration,
            visited_entries: 0,
        }
    }

    fn can_continue(&self) -> bool {
        self.visited_entries < self.max_entries && self.started_at.elapsed() <= self.max_duration
    }

    fn record_entry(&mut self) -> bool {
        if !self.can_continue() {
            return false;
        }
        self.visited_entries += 1;
        true
    }
}

fn find_file_by_name(
    root: &Path,
    file_name: &str,
    depth: usize,
    budget: &mut FileScanBudget,
) -> Option<PathBuf> {
    if depth == 0 || !budget.can_continue() {
        return None;
    }
    let entries = std::fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        if !budget.record_entry() {
            log::warn!(
                "[SessionTraces] 本地 Claude context 文件扫描达到上限 entries={} depth={}",
                budget.visited_entries,
                depth
            );
            return None;
        }
        let path = entry.path();
        if path.file_name().and_then(|value| value.to_str()) == Some(file_name) {
            return Some(path);
        }
        if path.is_dir() {
            if let Some(found) = find_file_by_name(&path, file_name, depth - 1, budget) {
                return Some(found);
            }
        }
    }
    None
}

fn strip_ansi(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for code in chars.by_ref() {
                if code.is_ascii_alphabetic() {
                    break;
                }
            }
            continue;
        }
        output.push(ch);
    }
    output
}

fn resolve_trace_session_title(
    conn: &rusqlite::Connection,
    where_clause: &str,
    params: &[&dyn rusqlite::ToSql],
) -> Option<String> {
    let sql = format!(
        "SELECT request_summary_json, request_json, response_text_preview
         FROM session_traces
         WHERE {where_clause}
         ORDER BY COALESCE(turn_index, 0) ASC, created_at ASC
         LIMIT 20"
    );
    let mut stmt = conn.prepare(&sql).ok()?;
    let mut rows = stmt.query(params).ok()?;
    while let Ok(Some(row)) = rows.next() {
        let summary: Option<String> = row.get(0).ok();
        let request: Option<String> = row.get(1).ok();
        let response: Option<String> = row.get(2).ok();
        if let Some(title) =
            title_from_trace_sources(summary.as_deref(), request.as_deref(), response.as_deref())
        {
            return Some(title);
        }
    }
    None
}

fn title_from_trace_sources(
    summary_json: Option<&str>,
    request_json: Option<&str>,
    response_preview: Option<&str>,
) -> Option<String> {
    summary_json
        .and_then(title_from_request_summary)
        .or_else(|| request_json.and_then(title_from_request_json))
        .or_else(|| response_preview.and_then(title_from_text_preview))
}

fn title_from_request_summary(summary_json: &str) -> Option<String> {
    let summary = serde_json::from_str::<Value>(summary_json).ok()?;
    title_from_request_summary_value(&summary)
}

fn title_from_request_summary_value(summary: &Value) -> Option<String> {
    summary
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn title_from_request_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<Value>(request_json).ok()?;
    request
        .get("messages")
        .or_else(|| request.get("input"))
        .and_then(first_user_text_from_messages)
        .map(|text| truncate_title(&text))
}

fn first_user_text_from_messages(messages: &Value) -> Option<String> {
    match messages {
        Value::Array(items) => items.iter().find_map(|item| {
            let role = item.get("role").and_then(Value::as_str).unwrap_or_default();
            if !role.eq_ignore_ascii_case("user") {
                return None;
            }
            extract_message_text(item)
                .map(|text| text.trim().to_string())
                .filter(|text| is_title_candidate(text))
        }),
        Value::String(text) => {
            let text = text.trim();
            is_title_candidate(text).then(|| text.to_string())
        }
        _ => None,
    }
}

fn extract_message_text(value: &Value) -> Option<String> {
    if let Some(text) = value.get("content").and_then(Value::as_str) {
        return Some(text.to_string());
    }
    if let Some(text) = value.get("text").and_then(Value::as_str) {
        return Some(text.to_string());
    }
    if let Some(content) = value.get("content").and_then(Value::as_array) {
        let parts = content
            .iter()
            .filter_map(|block| {
                block
                    .get("text")
                    .and_then(Value::as_str)
                    .or_else(|| block.get("content").and_then(Value::as_str))
                    .map(ToString::to_string)
            })
            .collect::<Vec<_>>();
        if !parts.is_empty() {
            return Some(parts.join("\n"));
        }
    }
    if let Some(content) = value
        .pointer("/message/content")
        .or_else(|| value.pointer("/input"))
    {
        return extract_message_text(content);
    }
    None
}

fn title_from_text_preview(text: &str) -> Option<String> {
    let line = text
        .lines()
        .map(str::trim)
        .find(|line| is_title_candidate(line))?;
    Some(truncate_title(line))
}

fn is_title_candidate(text: &str) -> bool {
    let text = text.trim();
    !text.is_empty()
        && !text.contains("<local-command-caveat>")
        && !text.contains("<command-name>")
        && !text.starts_with('/')
}

fn truncate_title(text: &str) -> String {
    const MAX_CHARS: usize = 80;
    let text = text.trim();
    if text.chars().count() <= MAX_CHARS {
        return text.to_string();
    }
    let truncated = text.chars().take(MAX_CHARS).collect::<String>();
    format!("{truncated}...(truncated)")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trace_with_context_stats(context_stats: Value) -> TraceTurnDetail {
        TraceTurnDetail {
            trace_id: "trace-1".to_string(),
            turn_index: Some(1),
            provider_id: Some("provider".to_string()),
            model: Some("claude".to_string()),
            request_model: Some("claude".to_string()),
            is_streaming: false,
            status_code: Some(200),
            system_prompt_preview: None,
            system_prompt_hash: None,
            message_count: 1,
            tool_count: 0,
            request_summary: json!({}),
            context_stats,
            context_window_tokens: None,
            context_used_tokens: None,
            context_usage_ratio: None,
            response_text_preview: None,
            tool_calls: json!([]),
            stop_reason: None,
            input_tokens: 0,
            output_tokens: 0,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
            latency_ms: None,
            first_token_ms: None,
            trace_mode: "summary".to_string(),
            created_at: 1,
        }
    }

    #[test]
    fn local_context_enrichment_marks_found_and_prevents_rescan() {
        let mut traces = vec![trace_with_context_stats(json!({}))];
        let context_text = "Context Usage\nMCP tools · /mcp\nLoaded\n├ mcp__demo__tool: 12 tokens";

        assert!(should_load_local_context_usage_text(
            Some("claude"),
            &traces
        ));
        let changed = apply_local_context_enrichment_to_traces(&mut traces, Some(context_text));

        assert!(changed);
        assert_eq!(traces[0].context_stats["resources"]["mcpTools"]["count"], 1);
        assert_eq!(
            traces[0].context_stats["localContextEnrichment"]["status"],
            "found"
        );
        assert!(!should_load_local_context_usage_text(
            Some("claude"),
            &traces
        ));
    }

    #[test]
    fn local_context_enrichment_marks_missing_and_prevents_rescan() {
        let mut traces = vec![trace_with_context_stats(json!({}))];

        assert!(should_load_local_context_usage_text(
            Some("claude"),
            &traces
        ));
        let changed = apply_local_context_enrichment_to_traces(&mut traces, None);

        assert!(changed);
        assert_eq!(
            traces[0].context_stats["localContextEnrichment"]["status"],
            "missing"
        );
        assert!(!should_load_local_context_usage_text(
            Some("claude"),
            &traces
        ));
    }

    #[test]
    fn local_context_file_search_respects_scan_budget() {
        let temp = tempfile::tempdir().expect("tempdir");
        let file_path = temp.path().join("session-1.jsonl");
        std::fs::write(&file_path, "{}\n").expect("write session file");

        let mut exhausted_budget = FileScanBudget::new(0, Duration::from_secs(60));
        assert_eq!(
            find_file_by_name(temp.path(), "session-1.jsonl", 4, &mut exhausted_budget),
            None
        );

        let mut budget = FileScanBudget::new(10, Duration::from_secs(60));
        assert_eq!(
            find_file_by_name(temp.path(), "session-1.jsonl", 4, &mut budget),
            Some(file_path)
        );
    }

    #[test]
    fn title_falls_back_to_legacy_request_json() {
        let request = json!({
            "messages": [
                {"role": "system", "content": "internal"},
                {"role": "user", "content": "请分析这个 Hermes 会话的质量"}
            ]
        });

        assert_eq!(
            title_from_trace_sources(
                Some("{}"),
                Some(&serde_json::to_string(&request).unwrap()),
                None,
            )
            .as_deref(),
            Some("请分析这个 Hermes 会话的质量")
        );
    }

    #[test]
    fn strip_ansi_keeps_context_markers_readable() {
        let text = "\u{1b}[1mContext Usage\u{1b}[22m\n\u{1b}[1mMCP tools\u{1b}[22m\u{1b}[38;2;153;153;153m · /mcp\u{1b}[39m";

        assert_eq!(strip_ansi(text), "Context Usage\nMCP tools · /mcp");
    }
}
