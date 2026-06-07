//! Session Traces data access layer.

use crate::error::AppError;

use super::super::{lock_conn, Database};

#[derive(Debug, Clone)]
pub(crate) struct SessionTraceInsert {
    pub trace_id: String,
    pub proxy_request_id: Option<String>,
    pub session_id: String,
    pub app_type: String,
    pub provider_id: Option<String>,
    pub model: Option<String>,
    pub request_model: Option<String>,
    pub is_streaming: bool,
    pub status_code: Option<u16>,
    pub system_prompt_preview: Option<String>,
    pub system_prompt_hash: Option<String>,
    pub message_count: u32,
    pub tool_count: u32,
    pub request_summary_json: String,
    pub context_stats_json: String,
    pub context_window_tokens: Option<u64>,
    pub context_used_tokens: Option<u64>,
    pub context_usage_ratio: Option<f64>,
    pub request_json: Option<String>,
    pub response_text_preview: Option<String>,
    pub response_text: Option<String>,
    pub response_json: Option<String>,
    pub tool_calls_json: String,
    pub stop_reason: Option<String>,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_read_tokens: u32,
    pub cache_creation_tokens: u32,
    pub latency_ms: Option<u64>,
    pub first_token_ms: Option<u64>,
    pub trace_mode: String,
    pub redaction_version: u32,
}

impl Database {
    pub(crate) fn insert_session_trace(&self, trace: &SessionTraceInsert) -> Result<(), AppError> {
        let now = chrono::Utc::now().timestamp();
        let conn = lock_conn!(self.conn);
        let turn_index: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(turn_index), 0) + 1
                 FROM session_traces
                 WHERE session_id = ?1 AND app_type = ?2",
                rusqlite::params![trace.session_id, trace.app_type],
                |row| row.get(0),
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute(
            "INSERT INTO session_traces (
                trace_id,
                proxy_request_id,
                session_id,
                turn_index,
                app_type,
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
                request_json,
                response_text_preview,
                response_text,
                response_json,
                tool_calls_json,
                stop_reason,
                input_tokens,
                output_tokens,
                cache_read_tokens,
                cache_creation_tokens,
                latency_ms,
                first_token_ms,
                trace_mode,
                redaction_version,
                created_at,
                updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20,
                ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30,
                ?31, ?32, ?33, ?34, ?35
            )",
            rusqlite::params![
                trace.trace_id,
                trace.proxy_request_id,
                trace.session_id,
                turn_index,
                trace.app_type,
                trace.provider_id,
                trace.model,
                trace.request_model,
                if trace.is_streaming { 1 } else { 0 },
                trace.status_code.map(i64::from),
                trace.system_prompt_preview,
                trace.system_prompt_hash,
                i64::from(trace.message_count),
                i64::from(trace.tool_count),
                trace.request_summary_json,
                trace.context_stats_json,
                trace.context_window_tokens.map(|v| v as i64),
                trace.context_used_tokens.map(|v| v as i64),
                trace.context_usage_ratio,
                trace.request_json,
                trace.response_text_preview,
                trace.response_text,
                trace.response_json,
                trace.tool_calls_json,
                trace.stop_reason,
                i64::from(trace.input_tokens),
                i64::from(trace.output_tokens),
                i64::from(trace.cache_read_tokens),
                i64::from(trace.cache_creation_tokens),
                trace.latency_ms.map(|v| v as i64),
                trace.first_token_ms.map(|v| v as i64),
                trace.trace_mode,
                i64::from(trace.redaction_version),
                now,
                now,
            ],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }
}
