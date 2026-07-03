//! Session trace capture helpers.
//!
//! The capture path is opt-in. When disabled, callers get `None` and do no
//! request/response JSON inspection for traces.

use super::{
    handler_config::UsageParserConfig,
    handler_context::RequestContext,
    server::ProxyState,
    usage::parser::{TokenUsage, SESSION_REQUEST_ID_PREFIX},
};
use crate::{
    database::SessionTraceInsert,
    settings::{SessionTraceMode, SessionTraceSettings},
};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Instant,
};
use tokio::sync::Mutex;

const REDACTED: &str = "[REDACTED]";
const MAX_CAPTURED_JSON_BYTES: usize = 256 * 1024;
const RESPONSE_EVENT_LIMIT: usize = 2_000;

#[derive(Debug, Clone)]
pub(crate) struct SessionTraceRequestSnapshot {
    trace_id: String,
    settings: SessionTraceSettings,
    system_prompt_preview: Option<String>,
    system_prompt_hash: Option<String>,
    message_count: u32,
    tool_count: u32,
    request_summary_json: String,
    context_stats_json: String,
    estimated_context_tokens: u64,
    context_window_tokens: Option<u64>,
    context_usage_ratio: Option<f64>,
    request_json: Option<String>,
}

pub(crate) fn build_request_snapshot(
    ctx: &RequestContext,
    body: &Value,
) -> Option<SessionTraceRequestSnapshot> {
    let mut settings = crate::settings::get_settings().session_traces;
    settings.normalize();
    if !settings.enabled || settings.mode == SessionTraceMode::Off {
        return None;
    }

    let system_prompt = extract_system_prompt(body);
    let system_prompt_preview = system_prompt
        .as_deref()
        .map(|text| truncate_chars(text, 1_000));
    let system_prompt_hash = system_prompt.as_deref().map(sha256_hex);

    let messages = body.get("messages").or_else(|| body.get("input"));
    let message_count = count_messages(messages);
    let tool_count = count_tools(body);
    let request_summary = build_request_summary(body, ctx, message_count, tool_count);
    let context_stats = build_context_stats(body, system_prompt.as_deref());
    let estimated_context_tokens = context_stats
        .get("totalTokens")
        .and_then(|v| v.as_u64())
        .unwrap_or_default();
    let context_window_tokens = infer_context_window_tokens(&ctx.request_model);
    let context_usage_ratio = context_window_tokens
        .filter(|window| *window > 0)
        .map(|window| estimated_context_tokens as f64 / window as f64);

    let request_json = if settings.mode == SessionTraceMode::Full && settings.capture_request_json {
        let value = if settings.redact_sensitive_values {
            redact_value(body)
        } else {
            body.clone()
        };
        serialize_bounded_json(&value)
    } else {
        None
    };

    Some(SessionTraceRequestSnapshot {
        trace_id: uuid::Uuid::new_v4().to_string(),
        settings,
        system_prompt_preview,
        system_prompt_hash,
        message_count,
        tool_count,
        request_summary_json: json_string(request_summary),
        context_stats_json: json_string(context_stats),
        estimated_context_tokens,
        context_window_tokens,
        context_usage_ratio,
        request_json,
    })
}

pub(crate) fn spawn_record_non_streaming_trace(
    state: &ProxyState,
    ctx: &RequestContext,
    snapshot: SessionTraceRequestSnapshot,
    status_code: u16,
    body_bytes: &[u8],
    parser_config: &UsageParserConfig,
) {
    let response_json = serde_json::from_slice::<Value>(body_bytes).ok();
    let usage = response_json
        .as_ref()
        .and_then(|value| (parser_config.response_parser)(value))
        .unwrap_or_default();
    let model = usage
        .model
        .clone()
        .or_else(|| {
            response_json
                .as_ref()
                .and_then(|value| value.get("model"))
                .and_then(|value| value.as_str())
                .map(ToString::to_string)
        })
        .unwrap_or_else(|| ctx.request_model.clone());
    let response_text = response_json.as_ref().and_then(extract_response_text);
    let response_text_preview = response_text
        .as_deref()
        .map(|text| truncate_chars(text, snapshot.settings.max_response_text_chars as usize));
    let response_json_string = if snapshot.settings.mode == SessionTraceMode::Full
        && snapshot.settings.capture_response_json
    {
        response_json
            .as_ref()
            .map(redact_value)
            .and_then(|value| serialize_bounded_json(&value))
    } else {
        None
    };
    let tool_calls_json = response_json
        .as_ref()
        .map(extract_tool_calls)
        .map(json_string)
        .unwrap_or_else(|| "[]".to_string());
    let stop_reason = response_json.as_ref().and_then(extract_stop_reason);

    spawn_insert_trace(
        state,
        ctx,
        snapshot,
        TraceCompletion {
            is_streaming: false,
            status_code,
            model,
            usage,
            response_text_preview,
            response_text: None,
            response_json: response_json_string,
            tool_calls_json,
            stop_reason,
            first_token_ms: None,
        },
    );
}

pub(crate) fn create_stream_trace_collector(
    state: &ProxyState,
    ctx: &RequestContext,
    snapshot: Option<SessionTraceRequestSnapshot>,
    status_code: u16,
    parser_config: &UsageParserConfig,
) -> Option<SseTraceCollector> {
    let snapshot = snapshot?;
    let state = state.clone();
    let provider_id = ctx.provider.id.clone();
    let request_model = ctx.request_model.clone();
    let original_model = ctx.original_model.clone();
    let app_type = ctx.app_type_str.to_string();
    let session_id = ctx.session_id.clone();
    let start_time = ctx.start_time;
    let stream_parser = parser_config.stream_parser;
    let model_extractor = parser_config.model_extractor;

    Some(SseTraceCollector::new(
        start_time,
        move |events, first_token_ms| {
            let usage = stream_parser(&events).unwrap_or_default();
            let model = model_extractor(&events, &request_model);
            let response_text = extract_stream_response_text(&events);
            let response_text_preview = response_text.as_deref().map(|text| {
                truncate_chars(text, snapshot.settings.max_response_text_chars as usize)
            });
            let response_json = if snapshot.settings.mode == SessionTraceMode::Full
                && snapshot.settings.capture_response_json
            {
                serialize_bounded_json(&redact_value(&Value::Array(events.clone())))
            } else {
                None
            };
            let tool_calls_json = json_string(extract_stream_tool_calls(&events));
            let stop_reason = extract_stream_stop_reason(&events);

            let state = state.clone();
            let provider_id = provider_id.clone();
            let request_model = request_model.clone();
            let original_model = original_model.clone();
            let app_type = app_type.clone();
            let session_id = session_id.clone();
            let snapshot = snapshot.clone();

            tokio::spawn(async move {
                let model = if original_model != request_model {
                    format!("vision_model -> {model}")
                } else {
                    model
                };
                insert_trace(
                    &state,
                    TraceInsertContext {
                        provider_id: Some(provider_id),
                        app_type,
                        session_id,
                        request_model,
                        latency_ms: start_time.elapsed().as_millis() as u64,
                    },
                    snapshot,
                    TraceCompletion {
                        is_streaming: true,
                        status_code,
                        model,
                        usage,
                        response_text_preview,
                        response_text: None,
                        response_json,
                        tool_calls_json,
                        stop_reason,
                        first_token_ms,
                    },
                );
            });
        },
    ))
}

#[derive(Clone)]
pub(crate) struct SseTraceCollector {
    inner: Arc<SseTraceCollectorInner>,
}

struct SseTraceCollectorInner {
    events: Mutex<Vec<Value>>,
    first_event_time: Mutex<Option<Instant>>,
    first_event_set: AtomicBool,
    start_time: Instant,
    on_complete: Arc<dyn Fn(Vec<Value>, Option<u64>) + Send + Sync + 'static>,
    finished: AtomicBool,
}

impl SseTraceCollector {
    fn new(
        start_time: Instant,
        callback: impl Fn(Vec<Value>, Option<u64>) + Send + Sync + 'static,
    ) -> Self {
        Self {
            inner: Arc::new(SseTraceCollectorInner {
                events: Mutex::new(Vec::new()),
                first_event_time: Mutex::new(None),
                first_event_set: AtomicBool::new(false),
                start_time,
                on_complete: Arc::new(callback),
                finished: AtomicBool::new(false),
            }),
        }
    }

    pub(crate) async fn push(&self, event: Value) {
        self.mark_first_event_time().await;
        let mut events = self.inner.events.lock().await;
        if events.len() < RESPONSE_EVENT_LIMIT {
            events.push(event);
        }
    }

    async fn mark_first_event_time(&self) {
        if self.inner.first_event_set.load(Ordering::Acquire) {
            return;
        }
        let mut first_time = self.inner.first_event_time.lock().await;
        if first_time.is_none() {
            *first_time = Some(Instant::now());
            self.inner.first_event_set.store(true, Ordering::Release);
        }
    }

    pub(crate) async fn finish(&self) {
        if self.inner.finished.swap(true, Ordering::SeqCst) {
            return;
        }

        let events = {
            let mut guard = self.inner.events.lock().await;
            std::mem::take(&mut *guard)
        };
        let first_token_ms = {
            let first_time = self.inner.first_event_time.lock().await;
            first_time.map(|time| (time - self.inner.start_time).as_millis() as u64)
        };
        (self.inner.on_complete)(events, first_token_ms);
    }
}

struct TraceCompletion {
    is_streaming: bool,
    status_code: u16,
    model: String,
    usage: TokenUsage,
    response_text_preview: Option<String>,
    response_text: Option<String>,
    response_json: Option<String>,
    tool_calls_json: String,
    stop_reason: Option<String>,
    first_token_ms: Option<u64>,
}

struct TraceInsertContext {
    provider_id: Option<String>,
    app_type: String,
    session_id: String,
    request_model: String,
    latency_ms: u64,
}

fn spawn_insert_trace(
    state: &ProxyState,
    ctx: &RequestContext,
    snapshot: SessionTraceRequestSnapshot,
    completion: TraceCompletion,
) {
    let insert_ctx = TraceInsertContext {
        provider_id: Some(ctx.provider.id.clone()),
        app_type: ctx.app_type_str.to_string(),
        session_id: ctx.session_id.clone(),
        request_model: ctx.request_model.clone(),
        latency_ms: ctx.latency_ms(),
    };
    let state = state.clone();
    tokio::spawn(async move {
        insert_trace(&state, insert_ctx, snapshot, completion);
    });
}

fn insert_trace(
    state: &ProxyState,
    insert_ctx: TraceInsertContext,
    snapshot: SessionTraceRequestSnapshot,
    completion: TraceCompletion,
) {
    let context_used_tokens = if completion.usage.input_tokens > 0
        || completion.usage.cache_read_tokens > 0
        || completion.usage.cache_creation_tokens > 0
    {
        u64::from(completion.usage.input_tokens)
            + u64::from(completion.usage.cache_read_tokens)
            + u64::from(completion.usage.cache_creation_tokens)
    } else {
        snapshot.estimated_context_tokens
    };
    let context_usage_ratio = snapshot
        .context_window_tokens
        .filter(|window| *window > 0)
        .map(|window| context_used_tokens as f64 / window as f64)
        .or(snapshot.context_usage_ratio);
    let proxy_request_id = completion
        .usage
        .message_id
        .as_ref()
        .map(|id| format!("{SESSION_REQUEST_ID_PREFIX}{id}"));

    let record = SessionTraceInsert {
        trace_id: snapshot.trace_id,
        proxy_request_id,
        session_id: insert_ctx.session_id,
        app_type: insert_ctx.app_type,
        provider_id: insert_ctx.provider_id,
        model: Some(completion.model),
        request_model: Some(insert_ctx.request_model),
        is_streaming: completion.is_streaming,
        status_code: Some(completion.status_code),
        system_prompt_preview: snapshot.system_prompt_preview,
        system_prompt_hash: snapshot.system_prompt_hash,
        message_count: snapshot.message_count,
        tool_count: snapshot.tool_count,
        request_summary_json: snapshot.request_summary_json,
        context_stats_json: snapshot.context_stats_json,
        context_window_tokens: snapshot.context_window_tokens,
        context_used_tokens: Some(context_used_tokens),
        context_usage_ratio,
        request_json: snapshot.request_json,
        response_text_preview: completion.response_text_preview,
        response_text: completion.response_text,
        response_json: completion.response_json,
        tool_calls_json: completion.tool_calls_json,
        stop_reason: completion.stop_reason,
        input_tokens: completion.usage.input_tokens,
        output_tokens: completion.usage.output_tokens,
        cache_read_tokens: completion.usage.cache_read_tokens,
        cache_creation_tokens: completion.usage.cache_creation_tokens,
        latency_ms: Some(insert_ctx.latency_ms),
        first_token_ms: completion.first_token_ms,
        trace_mode: trace_mode_label(&snapshot.settings.mode).to_string(),
        redaction_version: 1,
    };

    if let Err(err) = state.db.insert_session_trace(&record) {
        log::warn!("[SessionTraces] 写入 trace 失败: {err}");
    }
}

fn serialize_bounded_json(value: &Value) -> Option<String> {
    let serialized = serde_json::to_string(value).ok()?;
    if serialized.len() <= MAX_CAPTURED_JSON_BYTES {
        return Some(serialized);
    }

    Some(json_string(json!({
        "truncated": true,
        "reason": "session_trace_payload_too_large",
        "originalBytes": serialized.len(),
        "maxBytes": MAX_CAPTURED_JSON_BYTES,
    })))
}

fn build_request_summary(
    body: &Value,
    ctx: &RequestContext,
    message_count: u32,
    tool_count: u32,
) -> Value {
    let roles = summarize_roles(body.get("messages").or_else(|| body.get("input")));
    let content_types = summarize_content_types(body);
    let title = extract_session_title(body);
    json!({
        "appType": ctx.app_type_str,
        "requestModel": ctx.request_model,
        "originalModel": ctx.original_model,
        "title": title,
        "stream": body.get("stream").and_then(|v| v.as_bool()).unwrap_or(false),
        "messageCount": message_count,
        "toolCount": tool_count,
        "roles": roles,
        "contentTypes": content_types,
        "maxTokens": body.get("max_tokens").or_else(|| body.get("max_output_tokens")).cloned(),
        "temperature": body.get("temperature").cloned(),
    })
}

fn build_context_stats(body: &Value, system_prompt: Option<&str>) -> Value {
    let system_tokens = system_prompt.map(estimate_tokens).unwrap_or_default();
    let messages_tokens = body
        .get("messages")
        .or_else(|| body.get("input"))
        .map(estimate_tokens_from_value)
        .unwrap_or_default();
    let tools_tokens = body
        .get("tools")
        .or_else(|| body.get("functions"))
        .map(estimate_tokens_from_value)
        .unwrap_or_default();
    let other_tokens =
        estimate_other_request_tokens(body, system_tokens, messages_tokens, tools_tokens);
    let total_tokens = system_tokens + messages_tokens + tools_tokens + other_tokens;
    let context_text = collect_request_text(body, system_prompt);
    let resources = build_resource_inventory(body, &context_text);
    let parsed_context = parse_context_usage_text(&context_text);

    json!({
        "totalTokens": total_tokens,
        "categories": {
            "systemPrompt": system_tokens,
            "messages": messages_tokens,
            "tools": tools_tokens,
            "otherRequest": other_tokens
        },
        "resources": resources,
        "contextCommand": parsed_context,
        "estimator": "chars/4"
    })
}

pub(crate) fn enrich_context_stats_from_context_text(
    mut context_stats: Value,
    text: &str,
) -> Value {
    let resources = parse_context_resources(text);
    if let Some(inferred_resources) = resources.as_object().filter(|value| !value.is_empty()) {
        let Some(context_map) = context_stats.as_object_mut() else {
            return json!({
                "resources": Value::Object(inferred_resources.clone()),
                "contextCommand": parse_context_usage_text(text),
            });
        };
        let resources_value = context_map
            .entry("resources".to_string())
            .or_insert_with(|| json!({}));
        if let Some(resources) = resources_value.as_object_mut() {
            merge_resource_map(resources, Value::Object(inferred_resources.clone()));
        } else {
            *resources_value = Value::Object(inferred_resources.clone());
        }
    }

    if let Some(context_map) = context_stats.as_object_mut() {
        context_map.insert("contextCommand".to_string(), parse_context_usage_text(text));
    }
    context_stats
}

fn estimate_other_request_tokens(
    body: &Value,
    system_tokens: u64,
    messages_tokens: u64,
    tools_tokens: u64,
) -> u64 {
    estimate_tokens_from_value(body).saturating_sub(system_tokens + messages_tokens + tools_tokens)
}

fn redact_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut next = Map::new();
            for (key, value) in map {
                if is_sensitive_key(key) {
                    next.insert(key.clone(), Value::String(REDACTED.to_string()));
                } else {
                    next.insert(key.clone(), redact_value(value));
                }
            }
            Value::Object(next)
        }
        Value::Array(items) => Value::Array(items.iter().map(redact_value).collect()),
        _ => value.clone(),
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key.contains("key")
        || key.contains("token")
        || key.contains("secret")
        || key.contains("password")
        || key == "authorization"
        || key == "cookie"
}

fn extract_system_prompt(body: &Value) -> Option<String> {
    match body.get("system") {
        Some(Value::String(text)) => Some(text.clone()),
        Some(Value::Array(blocks)) => Some(
            blocks
                .iter()
                .filter_map(extract_text_from_content_block)
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .filter(|text| !text.trim().is_empty()),
        _ => body
            .get("instructions")
            .and_then(|value| value.as_str())
            .map(ToString::to_string),
    }
}

fn count_messages(messages: Option<&Value>) -> u32 {
    match messages {
        Some(Value::Array(items)) => items.len() as u32,
        Some(_) => 1,
        None => 0,
    }
}

fn count_tools(body: &Value) -> u32 {
    body.get("tools")
        .or_else(|| body.get("functions"))
        .and_then(|value| value.as_array())
        .map(|items| items.len() as u32)
        .unwrap_or_default()
}

fn summarize_roles(messages: Option<&Value>) -> Value {
    let mut counts = Map::new();
    if let Some(Value::Array(items)) = messages {
        for item in items {
            let role = item
                .get("role")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown");
            let current = counts
                .get(role)
                .and_then(|value| value.as_u64())
                .unwrap_or(0);
            counts.insert(role.to_string(), Value::from(current + 1));
        }
    }
    Value::Object(counts)
}

fn summarize_content_types(body: &Value) -> Value {
    let mut counts = Map::new();
    visit_content_types(body, &mut counts);
    Value::Object(counts)
}

fn extract_session_title(body: &Value) -> Option<String> {
    body.get("messages")
        .or_else(|| body.get("input"))
        .and_then(first_user_text_from_messages)
        .map(|text| truncate_chars(text.trim(), 80))
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
            .filter_map(extract_text_from_content_block)
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

fn is_title_candidate(text: &str) -> bool {
    let text = text.trim();
    !text.is_empty()
        && !text.contains("<local-command-caveat>")
        && !text.contains("<command-name>")
        && !text.starts_with('/')
}

fn collect_request_text(body: &Value, system_prompt: Option<&str>) -> String {
    let mut parts = Vec::new();
    if let Some(system_prompt) = system_prompt {
        parts.push(system_prompt.to_string());
    }
    collect_text_values(body, &mut parts);
    parts.join("\n")
}

fn collect_text_values(value: &Value, parts: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if matches!(key.as_str(), "text" | "content" | "instructions" | "system") {
                    if let Some(text) = value.as_str() {
                        parts.push(text.to_string());
                    }
                }
                collect_text_values(value, parts);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_text_values(item, parts);
            }
        }
        Value::String(text) if looks_like_context_text(text) => {
            parts.push(text.to_string());
        }
        _ => {}
    }
}

fn looks_like_context_text(text: &str) -> bool {
    text.contains("MCP tools · /mcp")
        || text.contains("Skills · /skills")
        || text.contains("Memory files · /memory")
        || text.contains("Custom agents · /agents")
        || text.contains("Context Usage")
}

fn build_resource_inventory(body: &Value, context_text: &str) -> Value {
    let parsed = parse_context_resources(context_text);
    let mut resources = parsed.as_object().cloned().unwrap_or_default();

    merge_resource_map(
        &mut resources,
        infer_embedded_context_resources(context_text),
    );

    merge_resource_map(&mut resources, infer_skill_usage_resources(context_text));

    let tool_resources = extract_request_tool_resources(body);
    if !tool_resources.is_empty() {
        let inferred = infer_tool_resources(&tool_resources);
        merge_resource_map(&mut resources, inferred);
    }

    Value::Object(resources)
}

fn merge_resource_map(target: &mut Map<String, Value>, source: Value) {
    let Some(source) = source.as_object() else {
        return;
    };
    for (key, value) in source {
        target.entry(key.clone()).or_insert_with(|| value.clone());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ToolResource {
    name: String,
    tokens: u64,
}

fn extract_request_tool_resources(body: &Value) -> Vec<ToolResource> {
    let mut resources = Vec::new();
    if let Some(raw_tools) = body
        .get("tools")
        .or_else(|| body.get("functions"))
        .and_then(|value| value.as_array())
    {
        for tool in raw_tools {
            if let Some(name) = tool
                .get("name")
                .and_then(|value| value.as_str())
                .or_else(|| {
                    tool.pointer("/function/name")
                        .and_then(|value| value.as_str())
                })
            {
                resources.push(ToolResource {
                    name: name.to_string(),
                    tokens: estimate_tokens_from_value(tool),
                });
            }
        }
    }
    resources.sort_by(|a, b| a.name.cmp(&b.name));
    resources.dedup_by(|a, b| a.name == b.name);
    resources
}

fn infer_tool_resources(tool_resources: &[ToolResource]) -> Value {
    let mut mcp_items = Vec::new();
    let mut plugin_items = Vec::new();
    let mut agent_items = Vec::new();
    let mut other_tool_items = Vec::new();

    for tool in tool_resources {
        let item = json!({
            "name": tool.name.clone(),
            "tokens": tool.tokens,
            "tokenLabel": format!("~{} tokens", tool.tokens),
            "source": "requestTools"
        });
        if tool.name.starts_with("mcp__") {
            mcp_items.push(item);
        } else if is_agent_tool_name(&tool.name) {
            agent_items.push(item);
        } else if tool.name.contains("__") {
            plugin_items.push(item);
        } else {
            other_tool_items.push(item);
        }
    }

    let mut resources = Map::new();
    if !mcp_items.is_empty() {
        resources.insert(
            "mcpTools".to_string(),
            resource_summary("MCP tools", vec![("Loaded", mcp_items)]),
        );
    }
    if !plugin_items.is_empty() {
        resources.insert(
            "plugins".to_string(),
            resource_summary("Plugins", vec![("Loaded", plugin_items)]),
        );
    }
    if !agent_items.is_empty() {
        resources.insert(
            "agentTools".to_string(),
            resource_summary("Agent tools", vec![("Loaded", agent_items)]),
        );
    }
    if !other_tool_items.is_empty() {
        resources.insert(
            "tools".to_string(),
            resource_summary("Tools", vec![("Loaded", other_tool_items)]),
        );
    }
    Value::Object(resources)
}

fn is_agent_tool_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "agent" | "task" | "subagent" | "delegate" | "spawn_agent"
    )
}

fn parse_context_resources(text: &str) -> Value {
    let mut resources = Map::new();
    for (key, title, marker) in [
        ("mcpTools", "MCP tools", "MCP tools · /mcp"),
        ("customAgents", "Custom agents", "Custom agents · /agents"),
        ("memoryFiles", "Memory files", "Memory files · /memory"),
        ("skills", "Skills", "Skills · /skills"),
    ] {
        let section = extract_context_section(text, marker);
        if !section.trim().is_empty() {
            resources.insert(key.to_string(), parse_resource_section(title, &section));
        }
    }
    Value::Object(resources)
}

fn infer_embedded_context_resources(text: &str) -> Value {
    let mut resources = Map::new();

    let memory_items = parse_embedded_memory_files(text);
    if !memory_items.is_empty() {
        resources.insert(
            "memoryFiles".to_string(),
            resource_summary("Memory files", vec![("Loaded", memory_items)]),
        );
    }

    let skill_groups = parse_markdown_table_sections(
        text,
        "## Available Skills",
        &[
            "## Project Overview",
            "## Development Commands",
            "## MCP Tools",
            "## Agents",
        ],
        "Skill",
    );
    if !skill_groups.is_empty() {
        resources.insert(
            "skills".to_string(),
            resource_summary(
                "Skills",
                skill_groups
                    .iter()
                    .map(|(name, items)| (name.as_str(), items.clone()))
                    .collect(),
            ),
        );
    }
    if !resources.contains_key("skills") {
        let skill_items = parse_available_skill_bullets(text);
        if !skill_items.is_empty() {
            resources.insert(
                "skills".to_string(),
                resource_summary("Skills", vec![("Loaded", skill_items)]),
            );
        }
    }

    let mcp_groups = parse_markdown_table_sections(
        text,
        "## MCP Tools",
        &["## Agents", "## Important Notes", "Contents of "],
        "Tool",
    );
    if !mcp_groups.is_empty() {
        resources.insert(
            "mcpTools".to_string(),
            resource_summary(
                "MCP tools",
                mcp_groups
                    .iter()
                    .map(|(name, items)| (name.as_str(), items.clone()))
                    .collect(),
            ),
        );
    }

    let agent_groups = parse_markdown_table_sections(
        text,
        "## Agents",
        &["## Important Notes", "Contents of "],
        "Agent",
    );
    if !agent_groups.is_empty() {
        resources.insert(
            "customAgents".to_string(),
            resource_summary(
                "Custom agents",
                agent_groups
                    .iter()
                    .map(|(name, items)| (name.as_str(), items.clone()))
                    .collect(),
            ),
        );
    }

    Value::Object(resources)
}

fn parse_embedded_memory_files(text: &str) -> Vec<Value> {
    let mut items = Vec::new();
    let marker = "Contents of ";
    let mut remaining = text;

    while let Some(start) = remaining.find(marker) {
        let after_marker = &remaining[start + marker.len()..];
        let Some((path, after_path)) = after_marker.split_once(":\n") else {
            break;
        };
        let path = path.split(" (").next().unwrap_or(path).trim().to_string();
        let next = after_path.find(marker).unwrap_or(after_path.len());
        let content = &after_path[..next];
        if looks_like_memory_path(&path) {
            let tokens = estimate_tokens(content);
            items.push(json!({
                "name": path,
                "tokens": tokens,
                "tokenLabel": format!("~{} tokens", tokens),
                "source": "embeddedContext"
            }));
        }
        remaining = &after_path[next..];
    }

    dedupe_resource_items(items)
}

fn looks_like_memory_path(path: &str) -> bool {
    path.contains("/.claude/rules/")
        || path.contains("/memory/")
        || path.ends_with("/CLAUDE.md")
        || path.ends_with("CLAUDE.md")
        || path.ends_with("/MEMORY.md")
        || path.ends_with("MEMORY.md")
}

fn parse_markdown_table_sections(
    text: &str,
    start_marker: &str,
    end_markers: &[&str],
    first_column_name: &str,
) -> Vec<(String, Vec<Value>)> {
    let section = extract_section_until(text, start_marker, end_markers);
    if section.trim().is_empty() {
        return Vec::new();
    }

    let mut groups: Vec<(String, Vec<Value>)> = Vec::new();
    let mut current_group = "Loaded".to_string();
    let mut current_items = Vec::new();
    let mut table_first_column: Option<usize> = None;

    for raw_line in section.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("### ") {
            if !current_items.is_empty() {
                groups.push((current_group.clone(), std::mem::take(&mut current_items)));
            }
            current_group = line.trim_start_matches("###").trim().to_string();
            table_first_column = None;
            continue;
        }
        if !line.starts_with('|') || line.contains("---") {
            continue;
        }
        let columns = markdown_table_columns(line);
        if columns.is_empty() {
            continue;
        }
        if columns
            .iter()
            .any(|column| column.eq_ignore_ascii_case(first_column_name))
        {
            table_first_column = columns
                .iter()
                .position(|column| column.eq_ignore_ascii_case(first_column_name));
            continue;
        }
        let Some(index) = table_first_column else {
            continue;
        };
        let Some(name) = columns.get(index).map(|value| clean_inline_code(value)) else {
            continue;
        };
        if name.is_empty() {
            continue;
        }
        let tokens = estimate_tokens(line);
        current_items.push(json!({
            "name": name,
            "tokens": tokens,
            "tokenLabel": format!("~{} tokens", tokens),
            "source": "embeddedContext"
        }));
    }

    if !current_items.is_empty() {
        groups.push((current_group, current_items));
    }

    groups
        .into_iter()
        .map(|(group, items)| (group, dedupe_resource_items(items)))
        .filter(|(_, items)| !items.is_empty())
        .collect()
}

fn parse_available_skill_bullets(text: &str) -> Vec<Value> {
    let lower = text.to_ascii_lowercase();
    let Some(start) = lower.find("available skills") else {
        return Vec::new();
    };
    let rest = &text[start + "available skills".len()..];
    let end = rest.find("\n## ").unwrap_or(rest.len());
    let section = &rest[..end];

    let items = section
        .lines()
        .filter_map(|raw_line| {
            let line = raw_line.trim();
            let item = line.strip_prefix("- ")?;
            let name = item
                .split(':')
                .next()
                .unwrap_or(item)
                .split(" - ")
                .next()
                .unwrap_or(item)
                .trim();
            let name = clean_inline_code(name);
            if !is_plausible_skill_name(&name) {
                return None;
            }
            let tokens = estimate_tokens(line);
            Some(json!({
                "name": name,
                "tokens": tokens,
                "tokenLabel": format!("~{} tokens", tokens),
                "source": "embeddedContext"
            }))
        })
        .collect();

    dedupe_resource_items(items)
}

fn is_plausible_skill_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().count() <= 120
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':' | '/' | '.'))
}

fn extract_section_until(text: &str, start_marker: &str, end_markers: &[&str]) -> String {
    let Some(start) = text.find(start_marker) else {
        return String::new();
    };
    let rest = &text[start + start_marker.len()..];
    let end = end_markers
        .iter()
        .filter_map(|marker| rest.find(marker))
        .min()
        .unwrap_or(rest.len());
    rest[..end].to_string()
}

fn markdown_table_columns(line: &str) -> Vec<String> {
    line.trim_matches('|')
        .split('|')
        .map(|column| column.trim().to_string())
        .collect()
}

fn clean_inline_code(value: &str) -> String {
    value
        .trim()
        .trim_matches('`')
        .trim_matches('*')
        .trim()
        .to_string()
}

fn infer_skill_usage_resources(text: &str) -> Value {
    let mut items = Vec::new();
    for raw_line in text.lines() {
        let line = raw_line.trim();
        for name in extract_used_skill_names(line) {
            items.push(json!({
                "name": name,
                "tokens": 0,
                "tokenLabel": "used",
                "source": "conversationUsage"
            }));
        }
    }

    let items = dedupe_resource_items(items);
    if items.is_empty() {
        return json!({});
    }

    json!({
        "usedSkills": resource_summary("Used skills", vec![("Used", items)])
    })
}

fn extract_used_skill_names(line: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut remaining = line;
    while let Some(start) = remaining.find("Skill(") {
        let after = &remaining[start + "Skill(".len()..];
        let Some((name, rest)) = after.split_once(')') else {
            break;
        };
        let name = clean_inline_code(name);
        if is_plausible_skill_name(&name) {
            names.push(name);
        }
        remaining = rest;
    }

    let lower = line.to_ascii_lowercase();
    if lower.starts_with("using ") && lower.contains(" skill") {
        if let Some(name) = extract_backticked_name(line) {
            if is_plausible_skill_name(&name) {
                names.push(name);
            }
        } else {
            let candidate = &line["using ".len()..];
            let lower_candidate = candidate.to_ascii_lowercase();
            if let Some(skill_index) = lower_candidate.find(" skill") {
                let name = clean_inline_code(&candidate[..skill_index]);
                if is_plausible_skill_name(&name) {
                    names.push(name);
                }
            }
        }
    }

    names
}

fn extract_backticked_name(line: &str) -> Option<String> {
    let (_, rest) = line.split_once('`')?;
    let (name, _) = rest.split_once('`')?;
    Some(clean_inline_code(name))
}

fn dedupe_resource_items(items: Vec<Value>) -> Vec<Value> {
    let mut seen = std::collections::BTreeSet::new();
    let mut deduped = Vec::new();
    for item in items {
        let name = item
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if name.is_empty() || !seen.insert(name) {
            continue;
        }
        deduped.push(item);
    }
    deduped
}

fn extract_context_section(text: &str, marker: &str) -> String {
    let Some(start) = text.find(marker) else {
        return String::new();
    };
    let rest = &text[start + marker.len()..];
    let next = [
        "MCP tools · /mcp",
        "Custom agents · /agents",
        "Memory files · /memory",
        "Skills · /skills",
    ]
    .iter()
    .filter(|candidate| **candidate != marker)
    .filter_map(|candidate| rest.find(candidate))
    .min()
    .unwrap_or(rest.len());
    rest[..next].to_string()
}

fn parse_resource_section(title: &str, section: &str) -> Value {
    let mut groups: Vec<(String, Vec<Value>)> = Vec::new();
    let mut current_group = "Loaded".to_string();
    let mut current_items: Vec<Value> = Vec::new();

    for raw_line in section.lines() {
        let line = normalize_tree_line(raw_line);
        if line.is_empty() {
            continue;
        }
        if is_resource_group_header(&line) {
            if !current_items.is_empty() {
                groups.push((current_group.clone(), std::mem::take(&mut current_items)));
            }
            current_group = line;
            continue;
        }
        if let Some(item) = parse_resource_item(&line) {
            current_items.push(item);
        }
    }

    if !current_items.is_empty() {
        groups.push((current_group, current_items));
    }

    let grouped = groups
        .iter()
        .map(|(name, items)| (name.as_str(), items.clone()))
        .collect::<Vec<_>>();
    resource_summary(title, grouped)
}

fn resource_summary(title: &str, groups: Vec<(&str, Vec<Value>)>) -> Value {
    let mut group_values = Map::new();
    let mut total_tokens = 0u64;
    let mut total_items = 0u64;

    for (name, items) in groups {
        total_items += items.len() as u64;
        total_tokens += items
            .iter()
            .filter_map(|item| item.get("tokens").and_then(|value| value.as_u64()))
            .sum::<u64>();
        group_values.insert(name.to_string(), Value::Array(items));
    }

    json!({
        "title": title,
        "count": total_items,
        "totalTokens": total_tokens,
        "groups": group_values
    })
}

fn normalize_tree_line(line: &str) -> String {
    line.trim()
        .trim_start_matches(['├', '└', '│', '─'])
        .trim()
        .to_string()
}

fn is_resource_group_header(line: &str) -> bool {
    !line.contains(':')
        && !line.contains("tokens")
        && !line.contains('·')
        && !line.contains('/')
        && line.chars().count() <= 40
}

fn parse_resource_item(line: &str) -> Option<Value> {
    let (name, rest) = line.split_once(':')?;
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let token_label = rest.trim().to_string();
    let tokens = parse_token_label(&token_label).unwrap_or(0);
    Some(json!({
        "name": name,
        "tokens": tokens,
        "tokenLabel": token_label
    }))
}

fn parse_context_usage_text(text: &str) -> Value {
    let mut categories = Map::new();
    let mut used_tokens = None;
    let mut window_tokens = None;
    let mut usage_ratio = None;
    let mut model = None;

    for raw_line in text.lines() {
        let line = normalize_context_line(raw_line);
        if line.is_empty() {
            continue;
        }
        if line.contains('/') && line.contains("tokens") && line.contains('%') {
            if let Some((used, window, ratio)) = parse_context_total_line(&line) {
                used_tokens = Some(used);
                window_tokens = Some(window);
                usage_ratio = Some(ratio);
            }
        } else if let Some((name, tokens, ratio)) = parse_context_category_line(&line) {
            categories.insert(
                name,
                json!({
                    "tokens": tokens,
                    "ratio": ratio
                }),
            );
        } else if model.is_none() && line.contains('[') && line.contains(']') {
            model = Some(line);
        }
    }

    json!({
        "model": model,
        "usedTokens": used_tokens,
        "windowTokens": window_tokens,
        "usageRatio": usage_ratio,
        "categories": categories
    })
}

fn normalize_context_line(line: &str) -> String {
    line.trim()
        .trim_start_matches(['⛀', '⛁', '⛶'])
        .trim()
        .to_string()
}

fn parse_context_total_line(line: &str) -> Option<(u64, u64, f64)> {
    let token_part = line.split("tokens").next()?.trim();
    let (used, window) = token_part.rsplit_once('/')?;
    let ratio = line
        .split('(')
        .nth(1)
        .and_then(|part| part.split('%').next())
        .and_then(|value| value.trim().parse::<f64>().ok())
        .map(|value| value / 100.0)?;
    Some((
        parse_compact_number(used.trim())?,
        parse_compact_number(window.trim())?,
        ratio,
    ))
}

fn parse_context_category_line(line: &str) -> Option<(String, u64, Option<f64>)> {
    let (name, rest) = line.split_once(':')?;
    if !rest.contains("tokens") {
        return None;
    }
    let token_label = rest.split("tokens").next()?.trim();
    let tokens = parse_compact_number(token_label)?;
    let ratio = rest
        .split('(')
        .nth(1)
        .and_then(|part| part.split('%').next())
        .and_then(|value| value.trim().parse::<f64>().ok())
        .map(|value| value / 100.0);
    Some((name.trim().to_string(), tokens, ratio))
}

fn parse_token_label(label: &str) -> Option<u64> {
    let normalized = label
        .replace("tokens", "")
        .replace("token", "")
        .replace(['~', '<'], "")
        .trim()
        .to_string();
    parse_compact_number(&normalized)
}

fn parse_compact_number(value: &str) -> Option<u64> {
    let value = value.trim().to_ascii_lowercase();
    if value.is_empty() {
        return None;
    }
    let (number, multiplier) = if let Some(number) = value.strip_suffix('k') {
        (number, 1_000.0)
    } else if let Some(number) = value.strip_suffix('m') {
        (number, 1_000_000.0)
    } else {
        (value.as_str(), 1.0)
    };
    number
        .trim()
        .parse::<f64>()
        .ok()
        .map(|number| (number * multiplier).round() as u64)
}

fn visit_content_types(value: &Value, counts: &mut Map<String, Value>) {
    match value {
        Value::Object(map) => {
            if let Some(content_type) = map.get("type").and_then(|value| value.as_str()) {
                let current = counts
                    .get(content_type)
                    .and_then(|value| value.as_u64())
                    .unwrap_or(0);
                counts.insert(content_type.to_string(), Value::from(current + 1));
            }
            for value in map.values() {
                visit_content_types(value, counts);
            }
        }
        Value::Array(items) => {
            for item in items {
                visit_content_types(item, counts);
            }
        }
        _ => {}
    }
}

fn extract_response_text(value: &Value) -> Option<String> {
    let mut parts = Vec::new();
    collect_response_text(value, &mut parts);
    (!parts.is_empty()).then(|| parts.join("\n"))
}

fn collect_response_text(value: &Value, parts: &mut Vec<String>) {
    if let Some(output_text) = value.get("output_text").and_then(|v| v.as_str()) {
        parts.push(output_text.to_string());
    }

    if let Some(content) = value.get("content").and_then(|v| v.as_array()) {
        for block in content {
            if let Some(text) = extract_text_from_content_block(block) {
                parts.push(text);
            }
        }
    }

    if let Some(output) = value.get("output").and_then(|v| v.as_array()) {
        for item in output {
            collect_response_text(item, parts);
        }
    }

    if let Some(choices) = value.get("choices").and_then(|v| v.as_array()) {
        for choice in choices {
            if let Some(text) = choice.pointer("/message/content").and_then(|v| v.as_str()) {
                parts.push(text.to_string());
            }
        }
    }

    if let Some(candidates) = value.get("candidates").and_then(|v| v.as_array()) {
        for candidate in candidates {
            if let Some(parts_value) = candidate
                .pointer("/content/parts")
                .and_then(|v| v.as_array())
            {
                for part in parts_value {
                    if let Some(text) = part.get("text").and_then(|v| v.as_str()) {
                        parts.push(text.to_string());
                    }
                }
            }
        }
    }
}

fn extract_text_from_content_block(value: &Value) -> Option<String> {
    value
        .get("text")
        .and_then(|text| text.as_str())
        .or_else(|| value.get("content").and_then(|text| text.as_str()))
        .map(ToString::to_string)
}

fn extract_tool_calls(value: &Value) -> Value {
    let mut calls = Vec::new();
    collect_tool_calls(value, &mut calls);
    Value::Array(calls)
}

fn collect_tool_calls(value: &Value, calls: &mut Vec<Value>) {
    match value {
        Value::Object(map) => {
            let type_name = map.get("type").and_then(|v| v.as_str());
            if matches!(type_name, Some("tool_use" | "function_call"))
                || map.contains_key("tool_calls")
            {
                calls.push(redact_value(value));
            }
            if let Some(tool_calls) = map.get("tool_calls").and_then(|v| v.as_array()) {
                for call in tool_calls {
                    calls.push(redact_value(call));
                }
            }
            for value in map.values() {
                collect_tool_calls(value, calls);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_tool_calls(item, calls);
            }
        }
        _ => {}
    }
}

fn extract_stop_reason(value: &Value) -> Option<String> {
    value
        .get("stop_reason")
        .and_then(|v| v.as_str())
        .or_else(|| {
            value
                .pointer("/choices/0/finish_reason")
                .and_then(|v| v.as_str())
        })
        .or_else(|| value.get("finish_reason").and_then(|v| v.as_str()))
        .or_else(|| value.get("status").and_then(|v| v.as_str()))
        .map(ToString::to_string)
}

fn extract_stream_response_text(events: &[Value]) -> Option<String> {
    let mut parts = Vec::new();
    for event in events {
        if let Some(text) = event.pointer("/delta/text").and_then(|v| v.as_str()) {
            parts.push(text.to_string());
        }
        if let Some(text) = event.get("delta").and_then(|v| v.as_str()) {
            parts.push(text.to_string());
        }
        if let Some(text) = event
            .pointer("/choices/0/delta/content")
            .and_then(|v| v.as_str())
        {
            parts.push(text.to_string());
        }
        if let Some(text) = event.get("text").and_then(|v| v.as_str()) {
            parts.push(text.to_string());
        }
        if let Some(response) = event.get("response") {
            if let Some(text) = extract_response_text(response) {
                parts.push(text);
            }
        }
        if let Some(item) = event.get("item") {
            if let Some(text) = extract_response_text(item) {
                parts.push(text);
            }
        }
    }
    (!parts.is_empty()).then(|| parts.join(""))
}

fn extract_stream_tool_calls(events: &[Value]) -> Value {
    let mut calls = Vec::new();
    for event in events {
        if let Value::Array(mut nested) = extract_tool_calls(event) {
            calls.append(&mut nested);
        }
    }
    Value::Array(calls)
}

fn extract_stream_stop_reason(events: &[Value]) -> Option<String> {
    events.iter().rev().find_map(|event| {
        extract_stop_reason(event)
            .or_else(|| {
                event
                    .pointer("/message/stop_reason")
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string)
            })
            .or_else(|| {
                event
                    .pointer("/response/status")
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string)
            })
    })
}

fn estimate_tokens_from_value(value: &Value) -> u64 {
    serde_json::to_string(value)
        .map(|text| estimate_tokens(&text))
        .unwrap_or_default()
}

fn estimate_tokens(text: &str) -> u64 {
    ((text.chars().count() as f64) / 4.0).ceil() as u64
}

fn infer_context_window_tokens(model: &str) -> Option<u64> {
    let model = model.to_ascii_lowercase();
    if model.contains("[1m]") || model.contains("1m") || model.contains("gemini") {
        Some(1_000_000)
    } else if model.contains("gpt-4.1") {
        Some(1_047_576)
    } else if model.contains("claude") || model.contains("o3") || model.contains("o4") {
        Some(200_000)
    } else {
        None
    }
}

fn truncate_chars(text: &str, max_chars: usize) -> String {
    let max_chars = max_chars.max(1);
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let truncated = text.chars().take(max_chars).collect::<String>();
    format!("{truncated}...(truncated)")
}

fn sha256_hex(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn json_string(value: Value) -> String {
    serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string())
}

fn trace_mode_label(mode: &SessionTraceMode) -> &'static str {
    match mode {
        SessionTraceMode::Off => "off",
        SessionTraceMode::Summary => "summary",
        SessionTraceMode::Full => "full",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redact_value_masks_nested_secrets() {
        let value = json!({
            "api_key": "abc",
            "nested": {"authorization": "Bearer token", "ok": true}
        });

        let redacted = redact_value(&value);

        assert_eq!(redacted["api_key"], REDACTED);
        assert_eq!(redacted["nested"]["authorization"], REDACTED);
        assert_eq!(redacted["nested"]["ok"], true);
    }

    #[test]
    fn extracts_claude_response_text() {
        let value = json!({
            "content": [
                {"type": "text", "text": "hello"},
                {"type": "text", "text": "world"}
            ]
        });

        assert_eq!(
            extract_response_text(&value).as_deref(),
            Some("hello\nworld")
        );
    }

    #[test]
    fn estimates_context_window_for_one_million_model() {
        assert_eq!(
            infer_context_window_tokens("claude-opus-4-8[1M]"),
            Some(1_000_000)
        );
    }

    #[test]
    fn extracts_session_title_from_first_real_user_message() {
        let body = json!({
            "messages": [
                {
                    "role": "user",
                    "content": "<local-command-caveat>generated command</local-command-caveat>"
                },
                {
                    "role": "user",
                    "content": "帮我优化 Session Traces 的标题和分类"
                }
            ]
        });

        assert_eq!(
            extract_session_title(&body).as_deref(),
            Some("帮我优化 Session Traces 的标题和分类")
        );
    }

    #[test]
    fn parses_context_resource_sections() {
        let text = r#"
MCP tools · /mcp

Loaded
├ mcp__mysql__list_tables: 0 tokens
└ mcp__context7__query-docs: 0 tokens

Custom agents · /agents

User
├ code-reviewer: 288 tokens
└ frontend-developer: 0 tokens

Memory files · /memory
├ ~/.claude/rules/coding-style.md: 595 tokens

Skills · /skills

Project
├ ayd-front: ~40 tokens
└ code-map: ~30 tokens

User
├ agent-browser: ~310 tokens
└ verify: < 20 tokens
"#;

        let resources = parse_context_resources(text);

        assert_eq!(resources["mcpTools"]["count"], 2);
        assert_eq!(resources["customAgents"]["totalTokens"], 288);
        assert_eq!(resources["memoryFiles"]["totalTokens"], 595);
        assert_eq!(resources["skills"]["count"], 4);
        assert_eq!(
            resources["skills"]["groups"]["User"][0]["name"],
            "agent-browser"
        );
    }

    #[test]
    fn infers_mcp_resources_from_request_tools() {
        let body = json!({
            "tools": [
                {"name": "Agent", "description": "Run a sub agent"},
                {"name": "mcp__mysql__list_tables", "description": "List tables"},
                {"name": "mcp__context7__query-docs", "description": "Query docs"},
                {"name": "str_replace_editor"}
            ]
        });

        let resources = build_resource_inventory(&body, "");

        assert_eq!(resources["mcpTools"]["count"], 2);
        assert_eq!(resources["agentTools"]["count"], 1);
        assert_eq!(resources["tools"]["count"], 1);
        assert!(resources["mcpTools"]["totalTokens"].as_u64().unwrap() > 0);
    }

    #[test]
    fn infers_embedded_claude_context_resources() {
        let body = json!({
            "messages": [{
                "role": "user",
                "content": [{
                    "type": "text",
                    "text": r#"
Contents of /Users/keane/.claude/rules/coding-style.md (user's private global instructions for all projects):

# Coding Style
Always validate input.

Contents of /Users/keane/www/ayd_company/Ayd/CLAUDE.md (project instructions, checked into the codebase):

## Available Skills

### AYD 核心 Skills（由 butler 自动调度）

| Skill | Description | 何时使用 |
|-------|-------------|----------|
| `ayd-butler` | **强制入口** | 每次对话必须调用 |
| `ayd-dev` | Laravel backend development | 由 butler 调度 |

## MCP Tools

### 自动调用（每次执行）

| Tool | 使用场景 | 调用方 |
|------|----------|--------|
| `sequential-thinking` | 需求分析 | ayd-butler |

### 后端自动调用

| Tool | 使用场景 | 调用方 |
|------|----------|--------|
| `mysql` | 数据库查看 | ayd-dev |

## Agents

### 自动调用（每次执行后）

| Agent | Use For | 调用方 |
|-------|---------|--------|
| `code-reviewer` | 代码质量审查 | ayd-dev |

Contents of /Users/keane/.claude/projects/-Users-keane-www-ayd-company-Ayd/memory/MEMORY.md (user's auto-memory, persists across conversations):

# Memory Index
- [必须调用 ayd-butler 入口](feedback_ayd_butler_entry.md)

⏺ Skill(ayd-butler)
  ⎿ Successfully loaded skill
"#
                }]
            }],
            "tools": [{"name": "Agent", "description": "Run a sub agent"}]
        });

        let stats = build_context_stats(&body, None);
        let resources = &stats["resources"];

        assert_eq!(resources["memoryFiles"]["count"], 3);
        assert_eq!(resources["skills"]["count"], 2);
        assert_eq!(resources["mcpTools"]["count"], 2);
        assert_eq!(resources["customAgents"]["count"], 1);
        assert_eq!(
            resources["usedSkills"]["groups"]["Used"][0]["name"],
            "ayd-butler"
        );
        assert_eq!(resources["agentTools"]["count"], 1);
    }

    #[test]
    fn infers_available_skills_from_bullet_list_context() {
        let body = json!({
            "messages": [{
                "role": "user",
                "content": r#"
### Available skills
- cc-gateway: CC Gateway Pro development and upstream sync.
- debugging: Systematic debugging workflow.
- source-command-tdd: Test-driven development workflow.

## MCP Tools
"#
            }]
        });

        let stats = build_context_stats(&body, None);
        let skills = &stats["resources"]["skills"];

        assert_eq!(skills["count"], 3);
        assert_eq!(skills["groups"]["Loaded"][0]["name"], "cc-gateway");
        assert_eq!(skills["groups"]["Loaded"][2]["name"], "source-command-tdd");
    }

    #[test]
    fn infers_used_skills_from_plain_using_messages() {
        let body = json!({
            "messages": [{
                "role": "assistant",
                "content": "Using `cc-gateway` skill for session trace constraints.\nUsing debugging skill to isolate the scan loop."
            }]
        });

        let stats = build_context_stats(&body, None);
        let used = &stats["resources"]["usedSkills"];

        assert_eq!(used["count"], 2);
        assert_eq!(used["groups"]["Used"][0]["name"], "cc-gateway");
        assert_eq!(used["groups"]["Used"][1]["name"], "debugging");
    }

    #[test]
    fn bounded_json_replaces_oversized_payload_with_marker() {
        let value = json!({
            "messages": [{
                "role": "user",
                "content": "x".repeat(MAX_CAPTURED_JSON_BYTES + 1)
            }]
        });

        let stored = serialize_bounded_json(&value).expect("serialize marker");
        let parsed: Value = serde_json::from_str(&stored).expect("valid json marker");

        assert!(stored.len() < 512);
        assert_eq!(parsed["truncated"], true);
        assert_eq!(parsed["reason"], "session_trace_payload_too_large");
        assert!(parsed["originalBytes"].as_u64().unwrap() > MAX_CAPTURED_JSON_BYTES as u64);
    }
}
