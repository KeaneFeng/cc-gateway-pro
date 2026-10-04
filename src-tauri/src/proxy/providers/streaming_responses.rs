//! OpenAI Responses API 流式转换模块
//!
//! 实现 Responses API SSE → Anthropic SSE 格式转换。
//!
//! Responses API 使用命名事件 (named events) 的生命周期模型：
//! response.created → output_item.added → content_part.added →
//! output_text.delta → content_part.done → output_item.done → response.completed
//!
//! 与 Chat Completions 的 delta chunk 模型完全不同，需要独立的状态机处理。

use super::reasoning_bridge::{encode_openai_reasoning_item, reasoning_summary_text};
use super::transform_responses::{
    build_anthropic_usage_from_responses, map_responses_stop_reason, responses_to_anthropic,
    sanitize_anthropic_tool_use_input_json,
};
use crate::proxy::sse::{strip_sse_field, take_sse_block};
use bytes::Bytes;
use futures::stream::{Stream, StreamExt};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

#[inline]
fn response_object_from_event(data: &Value) -> &Value {
    data.get("response").unwrap_or(data)
}

fn anthropic_sse(event_name: &str, payload: &Value) -> Bytes {
    Bytes::from(format!(
        "event: {event_name}\ndata: {}\n\n",
        serde_json::to_string(payload).unwrap_or_default()
    ))
}

fn responses_error_details(data: &Value, fallback: &str) -> (String, String) {
    let response = response_object_from_event(data);
    let error = response.get("error").unwrap_or(response);
    let message = error
        .get("message")
        .and_then(Value::as_str)
        .or_else(|| error.as_str())
        .filter(|message| !message.trim().is_empty())
        .unwrap_or(fallback)
        .to_string();
    let error_type = error
        .get("type")
        .and_then(Value::as_str)
        .or_else(|| error.get("code").and_then(Value::as_str))
        .unwrap_or("upstream_error")
        .to_string();
    (message, error_type)
}

fn anthropic_error_sse(message: &str, error_type: &str) -> Bytes {
    anthropic_sse(
        "error",
        &json!({
            "type": "error",
            "error": {"type": error_type, "message": message}
        }),
    )
}

fn anthropic_ping_sse() -> Bytes {
    anthropic_sse("ping", &json!({"type": "ping"}))
}

const UNPROCESSED_REJECTION_MESSAGE: &str = "Upstream rejected the request before processing it \
(terminal response.incomplete reported max_output_tokens with zero input/output usage). Real \
output-token truncation reports non-zero usage, so this is a request rejection — commonly caused \
by a tool schema the upstream refuses (e.g. a `pattern` regex with nested quantifiers), not by \
hitting the output-token limit.";

/// A terminal `incomplete` response that reports zero input and output usage
/// never ran on the upstream: genuine max-output-token truncation always
/// reports non-zero usage. The ChatGPT Codex backend produces exactly this
/// terminal (with `incomplete_details.reason = "max_output_tokens"`) when it
/// refuses a request before processing it, e.g. over a tool schema `pattern`
/// regex with nested quantifiers. Mapping such a terminal to
/// `stop_reason: "max_tokens"` makes clients report an output-token limit that
/// was never hit, so callers should surface it as an upstream error instead.
fn is_unprocessed_max_output_rejection(response_obj: &Value, status: Option<&str>) -> bool {
    if status != Some("incomplete") {
        return false;
    }
    if !matches!(
        response_obj
            .pointer("/incomplete_details/reason")
            .and_then(Value::as_str),
        Some("max_output_tokens") | Some("max_tokens") | None
    ) {
        return false;
    }
    response_obj
        .pointer("/usage/input_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        == 0
        && response_obj
            .pointer("/usage/output_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            == 0
}

/// Convert a compatible gateway's non-streaming Responses JSON into a complete
/// Anthropic SSE lifecycle. This is used when the client requested streaming but
/// the upstream ignored `stream:true` and returned `application/json`.
fn responses_json_to_anthropic_sse(
    body: Value,
    _hosted_web_search_name: Option<&str>,
    _max_web_search_uses: Option<u64>,
) -> Vec<Bytes> {
    // Zero-usage incomplete terminals are request rejections, not truncation;
    // decide before `body` is consumed by the conversion below.
    let unprocessed_rejection =
        is_unprocessed_max_output_rejection(&body, body.get("status").and_then(Value::as_str));
    let message = match responses_to_anthropic(body) {
        Ok(message) => message,
        Err(error) => {
            return vec![anthropic_error_sse(
                &error.to_string(),
                "response_transform_error",
            )]
        }
    };

    let usage = message.get("usage").cloned().unwrap_or_else(|| json!({}));
    let mut start_usage = usage.clone();
    start_usage["output_tokens"] = json!(0);
    let mut events = vec![anthropic_sse(
        "message_start",
        &json!({
            "type": "message_start",
            "message": {
                "id": message.get("id").cloned().unwrap_or_else(|| json!("")),
                "type": "message",
                "role": "assistant",
                "model": message.get("model").cloned().unwrap_or_else(|| json!("")),
                "usage": start_usage
            }
        }),
    )];

    if let Some(content) = message.get("content").and_then(Value::as_array) {
        for (index, block) in content.iter().enumerate() {
            let index = index as u64;
            match block.get("type").and_then(Value::as_str) {
                Some("text") => {
                    events.push(anthropic_sse(
                        "content_block_start",
                        &json!({"type":"content_block_start","index":index,"content_block":{"type":"text","text":""}}),
                    ));
                    if let Some(text) = block.get("text").and_then(Value::as_str) {
                        if !text.is_empty() {
                            events.push(anthropic_sse(
                                "content_block_delta",
                                &json!({"type":"content_block_delta","index":index,"delta":{"type":"text_delta","text":text}}),
                            ));
                        }
                    }
                    events.push(anthropic_sse(
                        "content_block_stop",
                        &json!({"type":"content_block_stop","index":index}),
                    ));
                }
                Some("tool_use") => {
                    events.push(anthropic_sse(
                        "content_block_start",
                        &json!({
                            "type":"content_block_start",
                            "index":index,
                            "content_block":{
                                "type":"tool_use",
                                "id":block.get("id").cloned().unwrap_or_else(|| json!("")),
                                "name":block.get("name").cloned().unwrap_or_else(|| json!("")),
                                "input":{}
                            }
                        }),
                    ));
                    let input = block.get("input").cloned().unwrap_or_else(|| json!({}));
                    events.push(anthropic_sse(
                        "content_block_delta",
                        &json!({
                            "type":"content_block_delta",
                            "index":index,
                            "delta":{"type":"input_json_delta","partial_json":serde_json::to_string(&input).unwrap_or_else(|_| "{}".to_string())}
                        }),
                    ));
                    events.push(anthropic_sse(
                        "content_block_stop",
                        &json!({"type":"content_block_stop","index":index}),
                    ));
                }
                Some("thinking") => {
                    events.push(anthropic_sse(
                        "content_block_start",
                        &json!({"type":"content_block_start","index":index,"content_block":{"type":"thinking","thinking":""}}),
                    ));
                    if let Some(thinking) = block.get("thinking").and_then(Value::as_str) {
                        if !thinking.is_empty() {
                            events.push(anthropic_sse(
                                "content_block_delta",
                                &json!({"type":"content_block_delta","index":index,"delta":{"type":"thinking_delta","thinking":thinking}}),
                            ));
                        }
                    }
                    if let Some(signature) = block.get("signature").and_then(Value::as_str) {
                        if !signature.is_empty() {
                            events.push(anthropic_sse(
                                "content_block_delta",
                                &json!({"type":"content_block_delta","index":index,"delta":{"type":"signature_delta","signature":signature}}),
                            ));
                        }
                    }
                    events.push(anthropic_sse(
                        "content_block_stop",
                        &json!({"type":"content_block_stop","index":index}),
                    ));
                }
                Some("redacted_thinking") => {
                    events.push(anthropic_sse(
                        "content_block_start",
                        &json!({"type":"content_block_start","index":index,"content_block":block}),
                    ));
                    events.push(anthropic_sse(
                        "content_block_stop",
                        &json!({"type":"content_block_stop","index":index}),
                    ));
                }
                _ => {}
            }
        }
    }

    if unprocessed_rejection
        && message
            .get("content")
            .and_then(Value::as_array)
            .is_none_or(|content| content.is_empty())
    {
        log::warn!(
            "[Claude/Responses] terminal incomplete with zero usage and no content; \
emitting api_error for the unprocessed upstream rejection instead of max_tokens"
        );
        events.push(anthropic_error_sse(
            UNPROCESSED_REJECTION_MESSAGE,
            "api_error",
        ));
        return events;
    }

    events.push(anthropic_sse(
        "message_delta",
        &json!({
            "type":"message_delta",
            "delta":{
                "stop_reason":message.get("stop_reason").cloned().unwrap_or(Value::Null),
                "stop_sequence":null
            },
            "usage":usage
        }),
    ));
    events.push(anthropic_sse(
        "message_stop",
        &json!({"type":"message_stop"}),
    ));
    events
}

#[inline]
fn content_part_key(data: &Value) -> Option<String> {
    if let (Some(item_id), Some(content_index)) = (
        data.get("item_id").and_then(|v| v.as_str()),
        data.get("content_index").and_then(|v| v.as_u64()),
    ) {
        return Some(format!("part:{item_id}:{content_index}"));
    }
    if let (Some(output_index), Some(content_index)) = (
        data.get("output_index").and_then(|v| v.as_u64()),
        data.get("content_index").and_then(|v| v.as_u64()),
    ) {
        return Some(format!("part:out:{output_index}:{content_index}"));
    }
    None
}

#[inline]
fn tool_item_key_from_added(data: &Value, item: &Value) -> Option<String> {
    if let Some(item_id) = item.get("id").and_then(|v| v.as_str()) {
        return Some(format!("tool:{item_id}"));
    }
    if let Some(item_id) = data.get("item_id").and_then(|v| v.as_str()) {
        return Some(format!("tool:{item_id}"));
    }
    if let Some(output_index) = data.get("output_index").and_then(|v| v.as_u64()) {
        return Some(format!("tool:out:{output_index}"));
    }
    None
}

#[inline]
fn tool_item_key_from_event(data: &Value) -> Option<String> {
    if let Some(item_id) = data.get("item_id").and_then(|v| v.as_str()) {
        return Some(format!("tool:{item_id}"));
    }
    if let Some(output_index) = data.get("output_index").and_then(|v| v.as_u64()) {
        return Some(format!("tool:out:{output_index}"));
    }
    None
}

#[inline]
fn reasoning_item_key(data: &Value, item: Option<&Value>) -> Option<String> {
    if let Some(item_id) = item
        .and_then(|value| value.get("id"))
        .and_then(Value::as_str)
        .or_else(|| data.get("item_id").and_then(Value::as_str))
    {
        return Some(format!("reasoning:{item_id}"));
    }
    data.get("output_index")
        .and_then(Value::as_u64)
        .map(|index| format!("reasoning:out:{index}"))
}

/// Resolve content index for a text/refusal content part event.
///
/// Uses `content_part_key` to look up or assign a stable index, falling back to
/// `fallback_open_index` when no key is available.
#[inline]
fn resolve_content_index(
    data: &Value,
    next_content_index: &mut u32,
    index_by_key: &mut HashMap<String, u32>,
    fallback_open_index: &mut Option<u32>,
) -> u32 {
    if let Some(k) = content_part_key(data) {
        if let Some(existing) = index_by_key.get(&k).copied() {
            existing
        } else {
            let assigned = *next_content_index;
            *next_content_index += 1;
            index_by_key.insert(k, assigned);
            assigned
        }
    } else if let Some(existing) = *fallback_open_index {
        existing
    } else {
        let assigned = *next_content_index;
        *next_content_index += 1;
        *fallback_open_index = Some(assigned);
        assigned
    }
}

/// 创建从 Responses API SSE 到 Anthropic SSE 的转换流
///
/// 状态机跟踪: message_id, current_model, has_sent_message_start, item/content index map
/// SSE 解析支持 named events (event: + data: 行)
pub fn create_anthropic_sse_stream_from_responses<E: std::error::Error + Send + 'static>(
    stream: impl Stream<Item = Result<Bytes, E>> + Send + 'static,
) -> impl Stream<Item = Result<Bytes, std::io::Error>> + Send {
    async_stream::stream! {
        let mut buffer = String::new();
        let mut utf8_remainder: Vec<u8> = Vec::new();
        let mut message_id: Option<String> = None;
        let mut current_model: Option<String> = None;
        let mut has_sent_message_start = false;
        let mut has_tool_use = false;
        let mut next_content_index: u32 = 0;
        let mut index_by_key: HashMap<String, u32> = HashMap::new();
        let mut open_indices: HashSet<u32> = HashSet::new();
        let mut fallback_open_index: Option<u32> = None;
        let mut current_text_index: Option<u32> = None;
        let mut tool_index_by_item_id: HashMap<String, u32> = HashMap::new();
        let mut tool_name_by_index: HashMap<u32, String> = HashMap::new();
        let mut tool_args_by_index: HashMap<u32, String> = HashMap::new();
        let mut tool_had_delta: HashSet<u32> = HashSet::new();
        let mut last_tool_index: Option<u32> = None;
        let mut reasoning_index_by_item_id: HashMap<String, u32> = HashMap::new();
        let mut reasoning_item_by_index: HashMap<u32, Value> = HashMap::new();
        let mut reasoning_text_by_index: HashMap<u32, String> = HashMap::new();
        let mut legacy_reasoning_index: Option<u32> = None;
        let mut has_substantive_output = false;
        let mut terminated = false;

        // Append an EOF sentinel so the same parser handles a final SSE event that
        // omitted its trailing blank line. The boolean distinguishes the sentinel
        // from a legitimate empty upstream chunk.
        let stream = stream
            .map(|result| (result, false))
            .chain(futures::stream::once(async {
                (Ok::<Bytes, E>(Bytes::new()), true)
            }));
        tokio::pin!(stream);

        while let Some((chunk, is_eof)) = stream.next().await {
            match chunk {
                Ok(bytes) => {
                    crate::proxy::sse::append_utf8_safe(&mut buffer, &mut utf8_remainder, &bytes);

                    // A few compatible gateways ignore stream:true and return one
                    // JSON document. Hold it intact until EOF, including any pretty-
                    // printed blank lines that would otherwise look like SSE separators.
                    let looks_like_json = matches!(
                        buffer
                            .trim_start_matches(|ch: char| ch.is_whitespace() || ch == '\u{feff}')
                            .as_bytes()
                            .first(),
                        Some(b'{') | Some(b'[')
                    );
                    if looks_like_json && !is_eof {
                        continue;
                    }
                    if looks_like_json && is_eof {
                        match serde_json::from_str::<Value>(buffer.trim()) {
                            Ok(body) => {
                                for event in responses_json_to_anthropic_sse(body, None, None) {
                                    yield Ok(event);
                                }
                                terminated = true;
                            }
                            Err(error) => {
                                yield Ok(anthropic_error_sse(
                                    &format!("Invalid JSON response from Responses upstream: {error}"),
                                    "response_parse_error",
                                ));
                                terminated = true;
                            }
                        }
                        buffer.clear();
                        continue;
                    }

                    if is_eof && !buffer.trim().is_empty() {
                        buffer.push_str("\n\n");
                    }

                    // SSE 事件由 \n\n 分隔
                    while let Some(block) = take_sse_block(&mut buffer) {
                        if block.trim().is_empty() {
                            continue;
                        }

                        // 解析 SSE 块：提取 event: 和 data: 行
                        let mut event_type: Option<String> = None;
                        let mut data_parts: Vec<String> = Vec::new();

                        for line in block.lines() {
                            if let Some(evt) = strip_sse_field(line, "event") {
                                event_type = Some(evt.trim().to_string());
                            } else if let Some(d) = strip_sse_field(line, "data") {
                                data_parts.push(d.to_string());
                            }
                        }

                        if data_parts.is_empty() {
                            continue;
                        }

                        let data_str = data_parts.join("\n");

                        // 解析 JSON 数据
                        let data: Value = match serde_json::from_str(&data_str) {
                            Ok(v) => v,
                            Err(_) => continue,
                        };

                        // Official streams use both a named SSE event and `type` in
                        // the JSON payload. Compatible gateways sometimes omit the
                        // `event:` line, so fall back to the payload type.
                        let event_name = event_type
                            .as_deref()
                            .filter(|name| !name.is_empty())
                            .or_else(|| data.get("type").and_then(Value::as_str))
                            .unwrap_or("");

                        log::debug!("[Claude/Responses] <<< SSE event: {event_name}");

                        // Ignore every event after a terminal response. In particular,
                        // do not synthesize message_start if a broken gateway emits a
                        // late delta after response.failed/error.
                        if terminated {
                            continue;
                        }

                        let delta_requires_message_start = matches!(
                            event_name,
                            "response.output_text.delta"
                                | "response.refusal.delta"
                                | "response.function_call_arguments.delta"
                                | "response.reasoning_summary_text.delta"
                                | "response.reasoning_text.delta"
                                | "response.reasoning.delta"
                        );
                        if delta_requires_message_start {
                            has_substantive_output = true;
                        }
                        if delta_requires_message_start && !has_sent_message_start {
                            yield Ok(anthropic_sse(
                                "message_start",
                                &json!({
                                    "type":"message_start",
                                    "message":{
                                        "id":message_id.clone().unwrap_or_default(),
                                        "type":"message",
                                        "role":"assistant",
                                        "model":current_model.clone().unwrap_or_default(),
                                        "usage":{"input_tokens":0,"output_tokens":0}
                                    }
                                }),
                            ));
                            has_sent_message_start = true;
                        }

                        match event_name {
                            // ================================================
                            // response.created → message_start
                            // ================================================
                            "response.created" => {
                                let response_obj = response_object_from_event(&data);
                                if let Some(id) = response_obj.get("id").and_then(|i| i.as_str()) {
                                    message_id = Some(id.to_string());
                                }
                                if let Some(model) =
                                    response_obj.get("model").and_then(|m| m.as_str())
                                {
                                    current_model = Some(model.to_string());
                                }

                                has_sent_message_start = true;
                                // Build usage with defensive null handling
                                // Some() wrapper ensures build function always receives valid input
                                // Fallback to empty object {} if usage field missing, ensuring message_start
                                // event always has valid usage structure for VSCode Extension compatibility
                                let start_usage = build_anthropic_usage_from_responses(
                                    Some(response_obj.get("usage").unwrap_or(&json!({}))),
                                );

                                let event = json!({
                                    "type": "message_start",
                                    "message": {
                                        "id": message_id.clone().unwrap_or_default(),
                                        "type": "message",
                                        "role": "assistant",
                                        "model": current_model.clone().unwrap_or_default(),
                                        "usage": start_usage
                                    }
                                });
                                let sse = format!("event: message_start\ndata: {}\n\n",
                                    serde_json::to_string(&event).unwrap_or_default());
                                log::debug!("[Claude/Responses] >>> Anthropic SSE: message_start");
                                yield Ok(Bytes::from(sse));
                            }

                            // ================================================
                            // response.content_part.added → content_block_start (text)
                            // ================================================
                            "response.content_part.added" => {
                                // 确保 message_start 已发送
                                if !has_sent_message_start {
                                    let start_event = json!({
                                        "type": "message_start",
                                        "message": {
                                            "id": message_id.clone().unwrap_or_default(),
                                            "type": "message",
                                            "role": "assistant",
                                            "model": current_model.clone().unwrap_or_default(),
                                            "usage": { "input_tokens": 0, "output_tokens": 0 }
                                        }
                                    });
                                    let sse = format!("event: message_start\ndata: {}\n\n",
                                        serde_json::to_string(&start_event).unwrap_or_default());
                                    yield Ok(Bytes::from(sse));
                                    has_sent_message_start = true;
                                }

                                if let Some(part) = data.get("part") {
                                    let part_type = part.get("type").and_then(|t| t.as_str());
                                    if matches!(part_type, Some("output_text") | Some("refusal")) {
                                        let index = if let Some(index) = current_text_index {
                                            index
                                        } else {
                                            let index = resolve_content_index(
                                                &data,
                                                &mut next_content_index,
                                                &mut index_by_key,
                                                &mut fallback_open_index,
                                            );
                                            current_text_index = Some(index);
                                            index
                                        };

                                        if open_indices.contains(&index) {
                                            continue;
                                        }

                                        let event = json!({
                                            "type": "content_block_start",
                                            "index": index,
                                            "content_block": {
                                                "type": "text",
                                                "text": ""
                                            }
                                        });
                                        let sse = format!("event: content_block_start\ndata: {}\n\n",
                                            serde_json::to_string(&event).unwrap_or_default());
                                        yield Ok(Bytes::from(sse));
                                        open_indices.insert(index);
                                    }
                                }
                            }

                            // ================================================
                            // response.output_text.delta → content_block_delta (text_delta)
                            // ================================================
                            "response.output_text.delta" => {
                                if let Some(delta) = data.get("delta").and_then(|d| d.as_str()) {
                                    let index = if let Some(index) = current_text_index {
                                        index
                                    } else {
                                        let index = resolve_content_index(
                                            &data,
                                            &mut next_content_index,
                                            &mut index_by_key,
                                            &mut fallback_open_index,
                                        );
                                        current_text_index = Some(index);
                                        index
                                    };

                                    if !open_indices.contains(&index) {
                                        let start_event = json!({
                                            "type": "content_block_start",
                                            "index": index,
                                            "content_block": {
                                                "type": "text",
                                                "text": ""
                                            }
                                        });
                                        let start_sse = format!("event: content_block_start\ndata: {}\n\n",
                                            serde_json::to_string(&start_event).unwrap_or_default());
                                        yield Ok(Bytes::from(start_sse));
                                        open_indices.insert(index);
                                    }
                                    let event = json!({
                                        "type": "content_block_delta",
                                        "index": index,
                                        "delta": {
                                            "type": "text_delta",
                                            "text": delta
                                        }
                                    });
                                    let sse = format!("event: content_block_delta\ndata: {}\n\n",
                                        serde_json::to_string(&event).unwrap_or_default());
                                    yield Ok(Bytes::from(sse));
                                }
                            }

                            // ================================================
                            // response.refusal.delta → content_block_delta (text_delta)
                            // ================================================
                            "response.refusal.delta" => {
                                if let Some(delta) = data.get("delta").and_then(|d| d.as_str()) {
                                    let index = if let Some(index) = current_text_index {
                                        index
                                    } else {
                                        let index = resolve_content_index(
                                            &data,
                                            &mut next_content_index,
                                            &mut index_by_key,
                                            &mut fallback_open_index,
                                        );
                                        current_text_index = Some(index);
                                        index
                                    };

                                    if !open_indices.contains(&index) {
                                        let start_event = json!({
                                            "type": "content_block_start",
                                            "index": index,
                                            "content_block": {
                                                "type": "text",
                                                "text": ""
                                            }
                                        });
                                        let start_sse = format!("event: content_block_start\ndata: {}\n\n",
                                            serde_json::to_string(&start_event).unwrap_or_default());
                                        yield Ok(Bytes::from(start_sse));
                                        open_indices.insert(index);
                                    }

                                    let event = json!({
                                        "type": "content_block_delta",
                                        "index": index,
                                        "delta": {
                                            "type": "text_delta",
                                            "text": delta
                                        }
                                    });
                                    let sse = format!("event: content_block_delta\ndata: {}\n\n",
                                        serde_json::to_string(&event).unwrap_or_default());
                                    yield Ok(Bytes::from(sse));
                                }
                            }

                            // ================================================
                            // response.content_part.done → content_block_stop
                            // ================================================
                            "response.content_part.done" => {}

                            // ================================================
                            // response.output_item.added (function_call) → content_block_start (tool_use)
                            // ================================================
                            "response.output_item.added" => {
                                if let Some(item) = data.get("item") {
                                    let item_type = item.get("type").and_then(|t| t.as_str()).unwrap_or("");
                                    if item_type == "function_call" {
                                        has_tool_use = true;
                                        has_substantive_output = true;
                                        if let Some(index) = current_text_index.take() {
                                            if open_indices.remove(&index) {
                                                let stop_event = json!({
                                                    "type": "content_block_stop",
                                                    "index": index
                                                });
                                                let stop_sse = format!("event: content_block_stop\ndata: {}\n\n",
                                                    serde_json::to_string(&stop_event).unwrap_or_default());
                                                yield Ok(Bytes::from(stop_sse));
                                            }
                                            if fallback_open_index == Some(index) {
                                                fallback_open_index = None;
                                            }
                                        }
                                        // 确保 message_start 已发送
                                        if !has_sent_message_start {
                                            let start_event = json!({
                                                "type": "message_start",
                                                "message": {
                                                    "id": message_id.clone().unwrap_or_default(),
                                                    "type": "message",
                                                    "role": "assistant",
                                                    "model": current_model.clone().unwrap_or_default(),
                                                    "usage": { "input_tokens": 0, "output_tokens": 0 }
                                                }
                                            });
                                            let sse = format!("event: message_start\ndata: {}\n\n",
                                                serde_json::to_string(&start_event).unwrap_or_default());
                                            yield Ok(Bytes::from(sse));
                                            has_sent_message_start = true;
                                        }

                                        let call_id = item.get("call_id").and_then(|i| i.as_str()).unwrap_or("");
                                        let name = item.get("name").and_then(|n| n.as_str()).unwrap_or("");
                                        let index = if let Some(k) = tool_item_key_from_added(&data, item) {
                                            if let Some(existing) = index_by_key.get(&k).copied() {
                                                existing
                                            } else {
                                                let assigned = next_content_index;
                                                next_content_index += 1;
                                                index_by_key.insert(k, assigned);
                                                assigned
                                            }
                                        } else {
                                            let assigned = next_content_index;
                                            next_content_index += 1;
                                            assigned
                                        };
                                        if let Some(item_id) = item
                                            .get("id")
                                            .and_then(|v| v.as_str())
                                            .or_else(|| data.get("item_id").and_then(|v| v.as_str()))
                                        {
                                            tool_index_by_item_id.insert(item_id.to_string(), index);
                                        }
                                        tool_name_by_index.insert(index, name.to_string());
                                        last_tool_index = Some(index);

                                        if open_indices.contains(&index) {
                                            continue;
                                        }

                                        tool_args_by_index.insert(index, String::new());

                                        let event = json!({
                                            "type": "content_block_start",
                                            "index": index,
                                            "content_block": {
                                                "type": "tool_use",
                                                "id": call_id,
                                                "name": name
                                            }
                                        });
                                        let sse = format!("event: content_block_start\ndata: {}\n\n",
                                            serde_json::to_string(&event).unwrap_or_default());
                                        yield Ok(Bytes::from(sse));
                                        open_indices.insert(index);
                                    } else if item_type == "reasoning" {
                                        if !has_sent_message_start {
                                            let start_event = json!({
                                                "type": "message_start",
                                                "message": {
                                                    "id": message_id.clone().unwrap_or_default(),
                                                    "type": "message",
                                                    "role": "assistant",
                                                    "model": current_model.clone().unwrap_or_default(),
                                                    "usage": { "input_tokens": 0, "output_tokens": 0 }
                                                }
                                            });
                                            let sse = format!("event: message_start\ndata: {}\n\n",
                                                serde_json::to_string(&start_event).unwrap_or_default());
                                            yield Ok(Bytes::from(sse));
                                            has_sent_message_start = true;
                                        }

                                        let index = if let Some(key) = reasoning_item_key(&data, Some(item)) {
                                            if let Some(existing) = index_by_key.get(&key).copied() {
                                                existing
                                            } else {
                                                let assigned = next_content_index;
                                                next_content_index += 1;
                                                index_by_key.insert(key, assigned);
                                                assigned
                                            }
                                        } else {
                                            let assigned = next_content_index;
                                            next_content_index += 1;
                                            assigned
                                        };
                                        if let Some(item_id) = item
                                            .get("id")
                                            .and_then(Value::as_str)
                                            .or_else(|| data.get("item_id").and_then(Value::as_str))
                                        {
                                            reasoning_index_by_item_id.insert(item_id.to_string(), index);
                                        }
                                        reasoning_item_by_index.insert(index, item.clone());
                                        reasoning_text_by_index.entry(index).or_default();
                                    }
                                    // message type output_item.added is handled via content_part.added
                                }
                            }

                            // ================================================
                            // response.function_call_arguments.delta → content_block_delta (input_json_delta)
                            // ================================================
                            "response.function_call_arguments.delta" => {
                                if let Some(delta) = data.get("delta").and_then(|d| d.as_str()) {
                                    has_tool_use = true;
                                    let item_id = data.get("item_id").and_then(|v| v.as_str());
                                    let index = if let Some(id) = item_id {
                                        tool_index_by_item_id.get(id).copied()
                                    } else {
                                        None
                                    }
                                    .or_else(|| {
                                        tool_item_key_from_event(&data)
                                            .and_then(|k| index_by_key.get(&k).copied())
                                    })
                                    .or(last_tool_index)
                                    .unwrap_or_else(|| {
                                        let assigned = next_content_index;
                                        next_content_index += 1;
                                        assigned
                                    });

                                    if let Some(id) = item_id {
                                        tool_index_by_item_id.insert(id.to_string(), index);
                                    }
                                    if let Some(name) = data.get("name").and_then(Value::as_str) {
                                        tool_name_by_index.insert(index, name.to_string());
                                    } else {
                                        tool_name_by_index.entry(index).or_default();
                                    }
                                    last_tool_index = Some(index);

                                    if !open_indices.contains(&index) {
                                        let start_event = json!({
                                            "type": "content_block_start",
                                            "index": index,
                                            "content_block": {
                                                "type": "tool_use",
                                                "id": data
                                                    .get("call_id")
                                                    .and_then(|v| v.as_str())
                                                    .or(item_id)
                                                    .unwrap_or(""),
                                                "name": data
                                                    .get("name")
                                                    .and_then(|v| v.as_str())
                                                    .unwrap_or("")
                                            }
                                        });
                                        let start_sse = format!("event: content_block_start\ndata: {}\n\n",
                                            serde_json::to_string(&start_event).unwrap_or_default());
                                        yield Ok(Bytes::from(start_sse));
                                        open_indices.insert(index);
                                    }

                                    tool_args_by_index
                                        .entry(index)
                                        .or_default()
                                        .push_str(delta);
                                    tool_had_delta.insert(index);

                                    if tool_name_by_index.get(&index).map(String::as_str) == Some("Read") {
                                        continue;
                                    }

                                    let event = json!({
                                        "type": "content_block_delta",
                                        "index": index,
                                        "delta": {
                                            "type": "input_json_delta",
                                            "partial_json": delta
                                        }
                                    });
                                    let sse = format!("event: content_block_delta\ndata: {}\n\n",
                                        serde_json::to_string(&event).unwrap_or_default());
                                    yield Ok(Bytes::from(sse));
                                }
                            }

                            // ================================================
                            // response.function_call_arguments.done → content_block_stop
                            // ================================================
                            "response.function_call_arguments.done" => {
                                has_tool_use = true;
                                let item_id = data.get("item_id").and_then(|v| v.as_str());
                                let index = if let Some(id) = item_id {
                                    tool_index_by_item_id.get(id).copied()
                                } else {
                                    None
                                }
                                .or_else(|| {
                                    tool_item_key_from_event(&data)
                                        .and_then(|k| index_by_key.get(&k).copied())
                                })
                                .or(last_tool_index);
                                if let Some(index) = index {
                                    if !open_indices.remove(&index) {
                                        continue;
                                    }
                                    if tool_name_by_index.get(&index).map(String::as_str) == Some("Read") {
                                        let raw = data
                                            .get("arguments")
                                            .or_else(|| data.pointer("/item/arguments"))
                                            .and_then(|v| v.as_str())
                                            .map(str::to_string)
                                            .unwrap_or_else(|| {
                                                tool_args_by_index
                                                    .get(&index)
                                                    .cloned()
                                                    .unwrap_or_default()
                                            });
                                        let sanitized = sanitize_anthropic_tool_use_input_json("Read", &raw);
                                        if !sanitized.is_empty() {
                                            let event = json!({
                                                "type": "content_block_delta",
                                                "index": index,
                                                "delta": {
                                                    "type": "input_json_delta",
                                                    "partial_json": sanitized
                                                }
                                            });
                                            let sse = format!("event: content_block_delta\ndata: {}\n\n",
                                                serde_json::to_string(&event).unwrap_or_default());
                                            yield Ok(Bytes::from(sse));
                                        }
                                    } else if !tool_had_delta.contains(&index) {
                                        // Some compatible gateways skip delta events and only
                                        // provide the complete arguments on the done event.
                                        if let Some(arguments) = data
                                            .get("arguments")
                                            .or_else(|| data.pointer("/item/arguments"))
                                            .and_then(Value::as_str)
                                            .filter(|value| !value.is_empty())
                                        {
                                            let event = json!({
                                                "type": "content_block_delta",
                                                "index": index,
                                                "delta": {
                                                    "type": "input_json_delta",
                                                    "partial_json": arguments
                                                }
                                            });
                                            let sse = format!("event: content_block_delta\ndata: {}\n\n",
                                                serde_json::to_string(&event).unwrap_or_default());
                                            yield Ok(Bytes::from(sse));
                                        }
                                    }
                                    let event = json!({
                                        "type": "content_block_stop",
                                        "index": index
                                    });
                                    let sse = format!("event: content_block_stop\ndata: {}\n\n",
                                        serde_json::to_string(&event).unwrap_or_default());
                                    yield Ok(Bytes::from(sse));
                                    if let Some(item_id) = item_id {
                                        tool_index_by_item_id.remove(item_id);
                                    }
                                    tool_name_by_index.remove(&index);
                                    tool_args_by_index.remove(&index);
                                    tool_had_delta.remove(&index);
                                }
                            }

                            // ================================================
                            // response.refusal.done → content_block_stop
                            // ================================================
                            "response.refusal.done" => {
                                let index = current_text_index.take().or_else(|| {
                                    let key = content_part_key(&data);
                                    if let Some(k) = key {
                                        index_by_key.get(&k).copied()
                                    } else {
                                        fallback_open_index
                                    }
                                });
                                if let Some(index) = index {
                                    if !open_indices.remove(&index) {
                                        continue;
                                    }
                                    let event = json!({
                                        "type": "content_block_stop",
                                        "index": index
                                    });
                                    let sse = format!("event: content_block_stop\ndata: {}\n\n",
                                        serde_json::to_string(&event).unwrap_or_default());
                                    yield Ok(Bytes::from(sse));
                                    if fallback_open_index == Some(index) {
                                        fallback_open_index = None;
                                    }
                                }
                            }

                            // ================================================
                            // Official reasoning text events → thinking_delta.
                            // response.reasoning.delta is kept as a compatibility alias.
                            // ================================================
                            "response.reasoning_summary_text.delta"
                            | "response.reasoning_text.delta"
                            | "response.reasoning.delta" => {
                                if let Some(delta) = data
                                    .get("delta")
                                    .or_else(|| data.get("text"))
                                    .and_then(|d| d.as_str())
                                {
                                    if let Some(index) = current_text_index.take() {
                                        if open_indices.remove(&index) {
                                            let stop_event = json!({
                                                "type": "content_block_stop",
                                                "index": index
                                            });
                                            let stop_sse = format!("event: content_block_stop\ndata: {}\n\n",
                                                serde_json::to_string(&stop_event).unwrap_or_default());
                                            yield Ok(Bytes::from(stop_sse));
                                        }
                                        if fallback_open_index == Some(index) {
                                            fallback_open_index = None;
                                        }
                                    }
                                    let item_id = data.get("item_id").and_then(Value::as_str);
                                    let item_key = reasoning_item_key(&data, None);
                                    let is_keyless = item_id.is_none() && item_key.is_none();
                                    let index = item_id
                                        .and_then(|id| reasoning_index_by_item_id.get(id).copied())
                                        .or_else(|| {
                                            item_key
                                                .as_ref()
                                                .and_then(|key| index_by_key.get(key).copied())
                                        })
                                        .or_else(|| {
                                            is_keyless
                                                .then_some(legacy_reasoning_index)
                                                .flatten()
                                        })
                                        .unwrap_or_else(|| {
                                            let assigned = next_content_index;
                                            next_content_index += 1;
                                            if let Some(key) = item_key {
                                                index_by_key.insert(key, assigned);
                                            }
                                            if let Some(id) = item_id {
                                                reasoning_index_by_item_id
                                                    .insert(id.to_string(), assigned);
                                            } else if is_keyless {
                                                legacy_reasoning_index = Some(assigned);
                                            }
                                            assigned
                                        });

                                    if !open_indices.contains(&index) {
                                        let start_event = json!({
                                            "type": "content_block_start",
                                            "index": index,
                                            "content_block": {
                                                "type": "thinking",
                                                "thinking": ""
                                            }
                                        });
                                        let start_sse = format!("event: content_block_start\ndata: {}\n\n",
                                            serde_json::to_string(&start_event).unwrap_or_default());
                                        yield Ok(Bytes::from(start_sse));
                                        open_indices.insert(index);
                                    }

                                    reasoning_text_by_index
                                        .entry(index)
                                        .or_default()
                                        .push_str(delta);

                                    let event = json!({
                                        "type": "content_block_delta",
                                        "index": index,
                                        "delta": {
                                            "type": "thinking_delta",
                                            "thinking": delta
                                        }
                                    });
                                    let sse = format!("event: content_block_delta\ndata: {}\n\n",
                                        serde_json::to_string(&event).unwrap_or_default());
                                    yield Ok(Bytes::from(sse));
                                }
                            }

                            // ================================================
                            // Official done events carry the complete visible text. If a
                            // gateway omitted deltas, emit the text here. The block stays
                            // open until output_item.done supplies encrypted_content.
                            // ================================================
                            "response.reasoning_summary_text.done"
                            | "response.reasoning_text.done" => {
                                let item_id = data.get("item_id").and_then(Value::as_str);
                                let item_key = reasoning_item_key(&data, None);
                                let index = item_id
                                    .and_then(|id| reasoning_index_by_item_id.get(id).copied())
                                    .or_else(|| {
                                        item_key
                                            .as_ref()
                                            .and_then(|key| index_by_key.get(key).copied())
                                    })
                                    .or_else(|| {
                                        (item_id.is_none() && item_key.is_none())
                                            .then_some(legacy_reasoning_index)
                                            .flatten()
                                    });
                                if let Some(index) = index {
                                    let already_emitted = reasoning_text_by_index
                                        .get(&index)
                                        .is_some_and(|value| !value.is_empty());
                                    if !already_emitted {
                                        if let Some(text) = data
                                            .get("text")
                                            .and_then(Value::as_str)
                                            .filter(|value| !value.is_empty())
                                        {
                                            if !open_indices.contains(&index) {
                                                let start_event = json!({
                                                    "type": "content_block_start",
                                                    "index": index,
                                                    "content_block": {"type": "thinking", "thinking": ""}
                                                });
                                                let start_sse = format!("event: content_block_start\ndata: {}\n\n",
                                                    serde_json::to_string(&start_event).unwrap_or_default());
                                                yield Ok(Bytes::from(start_sse));
                                                open_indices.insert(index);
                                            }
                                            reasoning_text_by_index
                                                .entry(index)
                                                .or_default()
                                                .push_str(text);
                                            let event = json!({
                                                "type": "content_block_delta",
                                                "index": index,
                                                "delta": {"type": "thinking_delta", "thinking": text}
                                            });
                                            let sse = format!("event: content_block_delta\ndata: {}\n\n",
                                                serde_json::to_string(&event).unwrap_or_default());
                                            yield Ok(Bytes::from(sse));
                                        }
                                    }
                                }
                            }

                            // Legacy gateways do not emit output_item.done, so retain the
                            // old close behavior for their non-standard done event.
                            "response.reasoning.done" => {
                                let item_id = data.get("item_id").and_then(Value::as_str);
                                let item_key = reasoning_item_key(&data, None);
                                let index = item_id
                                    .and_then(|id| reasoning_index_by_item_id.get(id).copied())
                                    .or_else(|| {
                                        item_key
                                            .as_ref()
                                            .and_then(|key| index_by_key.get(key).copied())
                                    })
                                    .or_else(|| {
                                        (item_id.is_none() && item_key.is_none())
                                            .then_some(legacy_reasoning_index)
                                            .flatten()
                                    });
                                if let Some(index) = index {
                                    if open_indices.remove(&index) {
                                        let event = json!({"type": "content_block_stop", "index": index});
                                        let sse = format!("event: content_block_stop\ndata: {}\n\n",
                                            serde_json::to_string(&event).unwrap_or_default());
                                        yield Ok(Bytes::from(sse));
                                    }
                                    if legacy_reasoning_index == Some(index) {
                                        legacy_reasoning_index = None;
                                    }
                                }
                            }

                            // ================================================
                            // response.completed / response.incomplete → message_delta + message_stop
                            // ================================================
                            "response.completed" | "response.incomplete" => {
                                let response_obj = response_object_from_event(&data);
                                if matches!(
                                    response_obj.get("status").and_then(Value::as_str),
                                    Some("failed" | "cancelled")
                                ) || response_obj
                                    .get("error")
                                    .is_some_and(|error| !error.is_null())
                                {
                                    let (message, error_type) = responses_error_details(
                                        &data,
                                        "Responses upstream returned a failed terminal response",
                                    );
                                    yield Ok(anthropic_error_sse(&message, &error_type));
                                    terminated = true;
                                    continue;
                                }
                                if !has_sent_message_start {
                                    if let Some(id) = response_obj.get("id").and_then(Value::as_str) {
                                        message_id = Some(id.to_string());
                                    }
                                    if let Some(model) =
                                        response_obj.get("model").and_then(Value::as_str)
                                    {
                                        current_model = Some(model.to_string());
                                    }
                                    yield Ok(anthropic_sse(
                                        "message_start",
                                        &json!({
                                            "type":"message_start",
                                            "message":{
                                                "id":message_id.clone().unwrap_or_default(),
                                                "type":"message",
                                                "role":"assistant",
                                                "model":current_model.clone().unwrap_or_default(),
                                                "usage":{"input_tokens":0,"output_tokens":0}
                                            }
                                        }),
                                    ));
                                    has_sent_message_start = true;
                                }
                                let terminal_status = response_obj
                                    .get("status")
                                    .and_then(Value::as_str)
                                    .or(match event_name {
                                        "response.incomplete" => Some("incomplete"),
                                        "response.completed" => Some("completed"),
                                        _ => None,
                                    });
                                let stop_reason = map_responses_stop_reason(
                                    terminal_status,
                                    has_tool_use,
                                    response_obj
                                        .pointer("/incomplete_details/reason")
                                        .and_then(|r| r.as_str()),
                                );

                                // Zero-usage max_tokens stops are upstream request
                                // rejections, not truncation; report them as an
                                // error instead of a fabricated limit hit.
                                if stop_reason == Some("max_tokens")
                                    && !has_substantive_output
                                    && is_unprocessed_max_output_rejection(
                                        response_obj,
                                        terminal_status,
                                    )
                                {
                                    log::warn!(
                                        "[Claude/Responses] terminal incomplete with zero usage \
    and no content; emitting api_error for the unprocessed upstream rejection instead of max_tokens"
                                    );
                                    yield Ok(anthropic_error_sse(
                                        UNPROCESSED_REJECTION_MESSAGE,
                                        "api_error",
                                    ));
                                    terminated = true;
                                    continue;
                                }

                                // Best effort: close any dangling blocks before message_delta/message_stop.
                                if !open_indices.is_empty() {
                                    let mut remaining: Vec<u32> = open_indices.iter().copied().collect();
                                    remaining.sort_unstable();
                                    for index in remaining {
                                        let stop_event = json!({
                                            "type": "content_block_stop",
                                            "index": index
                                        });
                                        let stop_sse = format!("event: content_block_stop\ndata: {}\n\n",
                                            serde_json::to_string(&stop_event).unwrap_or_default());
                                        yield Ok(Bytes::from(stop_sse));
                                        open_indices.remove(&index);
                                    }
                                }
                                fallback_open_index = None;

                                // Defensive: Always build usage_json, even if usage field missing
                                // Some() wrapper with fallback to {} ensures build_anthropic_usage_from_responses
                                // always receives valid input, preventing null pointer errors in VSCode Extension
                                let usage_json = build_anthropic_usage_from_responses(
                                    Some(response_obj.get("usage").unwrap_or(&json!({})))
                                );

                                // Emit message_delta (with usage + stop_reason)
                                let delta_event = json!({
                                    "type": "message_delta",
                                    "delta": {
                                        "stop_reason": stop_reason,
                                        "stop_sequence": null
                                    },
                                    "usage": usage_json
                                });
                                let sse = format!("event: message_delta\ndata: {}\n\n",
                                    serde_json::to_string(&delta_event).unwrap_or_default());
                                log::debug!("[Claude/Responses] >>> Anthropic SSE: message_delta");
                                yield Ok(Bytes::from(sse));

                                // Emit message_stop
                                let stop_event = json!({"type": "message_stop"});
                                let stop_sse = format!("event: message_stop\ndata: {}\n\n",
                                    serde_json::to_string(&stop_event).unwrap_or_default());
                                log::debug!("[Claude/Responses] >>> Anthropic SSE: message_stop");
                                yield Ok(Bytes::from(stop_sse));
                                terminated = true;
                            }

                            // ================================================
                            // Semantic failures can be carried inside an HTTP 2xx SSE.
                            // Preserve the upstream details instead of silently ending.
                            // ================================================
                            "response.failed" | "error" => {
                                let (message, error_type) = responses_error_details(
                                    &data,
                                    if event_name == "response.failed" {
                                        "Responses upstream reported response.failed"
                                    } else {
                                        "Responses upstream emitted an error event"
                                    },
                                );
                                yield Ok(anthropic_error_sse(&message, &error_type));
                                terminated = true;
                            }

                            // Lifecycle events that don't need Anthropic counterparts.
                            // Listed explicitly so new events trigger a match-completeness review.
                            "response.output_text.done" => {
                                if let Some(index) = current_text_index.take() {
                                    if open_indices.remove(&index) {
                                        let stop_event = json!({
                                            "type": "content_block_stop",
                                            "index": index
                                        });
                                        let stop_sse = format!("event: content_block_stop\ndata: {}\n\n",
                                            serde_json::to_string(&stop_event).unwrap_or_default());
                                        yield Ok(Bytes::from(stop_sse));
                                    }
                                    if fallback_open_index == Some(index) {
                                        fallback_open_index = None;
                                    }
                                }
                            }
                            "response.output_item.done" => {
                                let Some(item) = data.get("item") else {
                                    continue;
                                };
                                match item.get("type").and_then(Value::as_str) {
                                    Some("function_call") => {
                                        has_tool_use = true;
                                        let item_id = item
                                            .get("id")
                                            .and_then(Value::as_str)
                                            .or_else(|| data.get("item_id").and_then(Value::as_str));
                                        let index = item_id
                                            .and_then(|id| tool_index_by_item_id.get(id).copied())
                                            .or_else(|| {
                                                tool_item_key_from_event(&data)
                                                    .and_then(|key| index_by_key.get(&key).copied())
                                            })
                                            .or(last_tool_index);
                                        if let Some(index) = index.filter(|value| open_indices.contains(value)) {
                                            let name = tool_name_by_index
                                                .get(&index)
                                                .map(String::as_str)
                                                .unwrap_or("");
                                            if !tool_had_delta.contains(&index) || name == "Read" {
                                                let raw = item
                                                    .get("arguments")
                                                    .and_then(Value::as_str)
                                                    .filter(|value| !value.is_empty())
                                                    .map(str::to_string)
                                                    .unwrap_or_else(|| {
                                                        tool_args_by_index
                                                            .get(&index)
                                                            .cloned()
                                                            .unwrap_or_default()
                                                    });
                                                let arguments = if name == "Read" {
                                                    sanitize_anthropic_tool_use_input_json(name, &raw)
                                                } else {
                                                    raw
                                                };
                                                if !arguments.is_empty() {
                                                    let event = json!({
                                                        "type": "content_block_delta",
                                                        "index": index,
                                                        "delta": {
                                                            "type": "input_json_delta",
                                                            "partial_json": arguments
                                                        }
                                                    });
                                                    let sse = format!("event: content_block_delta\ndata: {}\n\n",
                                                        serde_json::to_string(&event).unwrap_or_default());
                                                    yield Ok(Bytes::from(sse));
                                                }
                                            }
                                            open_indices.remove(&index);
                                            let event = json!({"type": "content_block_stop", "index": index});
                                            let sse = format!("event: content_block_stop\ndata: {}\n\n",
                                                serde_json::to_string(&event).unwrap_or_default());
                                            yield Ok(Bytes::from(sse));
                                            if let Some(id) = item_id {
                                                tool_index_by_item_id.remove(id);
                                            }
                                            tool_name_by_index.remove(&index);
                                            tool_args_by_index.remove(&index);
                                            tool_had_delta.remove(&index);
                                        }
                                    }
                                    Some("reasoning") => {
                                        let item_id = item
                                            .get("id")
                                            .and_then(Value::as_str)
                                            .or_else(|| data.get("item_id").and_then(Value::as_str));
                                        let index = item_id
                                            .and_then(|id| reasoning_index_by_item_id.get(id).copied())
                                            .or_else(|| {
                                                reasoning_item_key(&data, Some(item))
                                                    .and_then(|key| index_by_key.get(&key).copied())
                                            })
                                            .unwrap_or_else(|| {
                                                let assigned = next_content_index;
                                                next_content_index += 1;
                                                assigned
                                            });
                                        reasoning_item_by_index.insert(index, item.clone());

                                        let final_item = reasoning_item_by_index
                                            .get(&index)
                                            .cloned()
                                            .unwrap_or_else(|| item.clone());
                                        let full_text = reasoning_summary_text(&final_item);
                                        let emitted_text = reasoning_text_by_index
                                            .get(&index)
                                            .cloned()
                                            .unwrap_or_default();
                                        if emitted_text.is_empty() && !full_text.is_empty() {
                                            let start_event = json!({
                                                "type": "content_block_start",
                                                "index": index,
                                                "content_block": {"type": "thinking", "thinking": ""}
                                            });
                                            let start_sse = format!("event: content_block_start\ndata: {}\n\n",
                                                serde_json::to_string(&start_event).unwrap_or_default());
                                            yield Ok(Bytes::from(start_sse));
                                            open_indices.insert(index);
                                            let delta_event = json!({
                                                "type": "content_block_delta",
                                                "index": index,
                                                "delta": {"type": "thinking_delta", "thinking": full_text}
                                            });
                                            let delta_sse = format!("event: content_block_delta\ndata: {}\n\n",
                                                serde_json::to_string(&delta_event).unwrap_or_default());
                                            yield Ok(Bytes::from(delta_sse));
                                        }

                                        let encrypted = final_item
                                            .get("encrypted_content")
                                            .and_then(Value::as_str)
                                            .is_some_and(|value| !value.is_empty());
                                        if encrypted {
                                            if let Some(envelope) = encode_openai_reasoning_item(&final_item) {
                                                if open_indices.contains(&index) {
                                                    let signature_event = json!({
                                                        "type": "content_block_delta",
                                                        "index": index,
                                                        "delta": {
                                                            "type": "signature_delta",
                                                            "signature": envelope
                                                        }
                                                    });
                                                    let signature_sse = format!("event: content_block_delta\ndata: {}\n\n",
                                                        serde_json::to_string(&signature_event).unwrap_or_default());
                                                    yield Ok(Bytes::from(signature_sse));
                                                } else {
                                                    let start_event = json!({
                                                        "type": "content_block_start",
                                                        "index": index,
                                                        "content_block": {
                                                            "type": "redacted_thinking",
                                                            "data": envelope
                                                        }
                                                    });
                                                    let start_sse = format!("event: content_block_start\ndata: {}\n\n",
                                                        serde_json::to_string(&start_event).unwrap_or_default());
                                                    yield Ok(Bytes::from(start_sse));
                                                    open_indices.insert(index);
                                                }
                                            }
                                        }
                                        if open_indices.remove(&index) {
                                            let stop_event = json!({"type": "content_block_stop", "index": index});
                                            let stop_sse = format!("event: content_block_stop\ndata: {}\n\n",
                                                serde_json::to_string(&stop_event).unwrap_or_default());
                                            yield Ok(Bytes::from(stop_sse));
                                        }
                                        if let Some(id) = item_id {
                                            reasoning_index_by_item_id.remove(id);
                                        }
                                        reasoning_item_by_index.remove(&index);
                                        reasoning_text_by_index.remove(&index);
                                    }
                                    _ => {}
                                }
                            }
                            "response.reasoning_summary_part.added"
                            | "response.reasoning_summary_part.done"
                            | "response.in_progress" => {}

                            // Any other unknown/future events — silently skip.
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    log::error!("Responses stream error: {e}");
                    let error_event = json!({
                        "type": "error",
                        "error": {
                            "type": "stream_error",
                            "message": format!("Stream error: {e}")
                        }
                    });
                    let sse = format!("event: error\ndata: {}\n\n",
                        serde_json::to_string(&error_event).unwrap_or_default());
                    yield Ok(Bytes::from(sse));
                    terminated = true;
                    break;
                }
            }
        }

        if !terminated {
            let has_open_tool = open_indices.iter().any(|index| {
                tool_name_by_index.contains_key(index) || tool_args_by_index.contains_key(index)
            });
            let has_open_reasoning = open_indices.iter().any(|index| {
                reasoning_item_by_index.contains_key(index)
                    || reasoning_text_by_index.contains_key(index)
                    || legacy_reasoning_index == Some(*index)
            });

            if has_substantive_output && !has_open_tool && !has_open_reasoning {
                // Text-only partial output is safe to expose as a max-token style
                // incomplete turn. Close blocks before the terminal events.
                let mut remaining: Vec<u32> = open_indices.iter().copied().collect();
                remaining.sort_unstable();
                for index in remaining {
                    yield Ok(anthropic_sse(
                        "content_block_stop",
                        &json!({"type":"content_block_stop","index":index}),
                    ));
                }
                if !has_sent_message_start {
                    yield Ok(anthropic_sse(
                        "message_start",
                        &json!({
                            "type":"message_start",
                            "message":{
                                "id":message_id.clone().unwrap_or_default(),
                                "type":"message",
                                "role":"assistant",
                                "model":current_model.clone().unwrap_or_default(),
                                "usage":{"input_tokens":0,"output_tokens":0}
                            }
                        }),
                    ));
                }
                yield Ok(anthropic_sse(
                    "message_delta",
                    &json!({
                        "type":"message_delta",
                        "delta":{"stop_reason":"max_tokens","stop_sequence":null},
                        "usage":{"input_tokens":0,"output_tokens":0}
                    }),
                ));
                yield Ok(anthropic_sse("message_stop", &json!({"type":"message_stop"})));
            } else {
                // A truncated tool/reasoning block cannot be safely finalized: tool
                // JSON may be partial and thinking may be missing its signature.
                yield Ok(anthropic_error_sse(
                    "Responses upstream stream ended before a terminal event",
                    "stream_truncated",
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream;
    use futures::StreamExt;
    use std::collections::HashMap;

    async fn convert_stream_text(input: impl Into<Bytes>) -> String {
        let upstream = stream::iter(vec![Ok::<_, std::io::Error>(input.into())]);
        create_anthropic_sse_stream_from_responses(upstream)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(|chunk| String::from_utf8_lossy(chunk.unwrap().as_ref()).to_string())
            .collect()
    }

    #[test]
    fn test_map_responses_stop_reason_tool_use() {
        assert_eq!(
            map_responses_stop_reason(Some("completed"), true, None),
            Some("tool_use")
        );
        assert_eq!(
            map_responses_stop_reason(Some("completed"), false, None),
            Some("end_turn")
        );
        assert_eq!(
            map_responses_stop_reason(Some("incomplete"), false, Some("max_output_tokens")),
            Some("max_tokens")
        );
        assert_eq!(
            map_responses_stop_reason(Some("incomplete"), false, Some("content_filter")),
            Some("end_turn")
        );
    }

    #[test]
    fn test_response_object_from_event_with_wrapper() {
        let data = json!({
            "type": "response.created",
            "response": {
                "id": "resp_1",
                "model": "gpt-4o"
            }
        });
        let obj = response_object_from_event(&data);
        assert_eq!(obj["id"], "resp_1");
        assert_eq!(obj["model"], "gpt-4o");
    }

    #[tokio::test]
    async fn test_response_failed_event_becomes_anthropic_error() {
        let input = concat!(
            "event: response.created\n",
            "data: {\"type\":\"response.created\",\"response\":{\"id\":\"resp_1\",\"model\":\"gpt-5\"}}\n\n",
            "event: response.failed\n",
            "data: {\"type\":\"response.failed\",\"response\":{\"status\":\"failed\",\"error\":{\"type\":\"server_error\",\"message\":\"backend exploded\"}}}\n\n"
        );

        let merged = convert_stream_text(input).await;
        assert!(merged.contains("event: error"));
        assert!(merged.contains("backend exploded"));
        assert!(!merged.contains("event: message_stop"));
    }

    #[tokio::test]
    async fn test_late_delta_after_failure_does_not_emit_message_start() {
        let input = concat!(
            "event: response.failed\n",
            "data: {\"type\":\"response.failed\",\"response\":{\"status\":\"failed\",\"error\":{\"message\":\"boom\"}}}\n\n",
            "event: response.output_text.delta\n",
            "data: {\"type\":\"response.output_text.delta\",\"delta\":\"too late\"}\n\n"
        );

        let merged = convert_stream_text(input).await;
        assert!(merged.contains("event: error"));
        assert!(!merged.contains("event: message_start"));
        assert!(!merged.contains("too late"));
    }

    #[tokio::test]
    async fn test_completed_event_with_failed_status_is_error() {
        let input = concat!(
            "event: response.completed\n",
            "data: {\"type\":\"response.completed\",\"response\":{\"status\":\"failed\",\"error\":{\"type\":\"server_error\",\"message\":\"failed wrapper\"},\"output\":[]}}\n\n"
        );

        let merged = convert_stream_text(input).await;
        assert!(merged.contains("event: error"));
        assert!(merged.contains("failed wrapper"));
        assert!(!merged.contains("event: message_stop"));
    }

    #[tokio::test]
    async fn test_response_incomplete_event_terminates_with_max_tokens() {
        let input = concat!(
            "event: response.created\n",
            "data: {\"type\":\"response.created\",\"response\":{\"id\":\"resp_1\",\"model\":\"gpt-5\"}}\n\n",
            "event: response.incomplete\n",
            "data: {\"type\":\"response.incomplete\",\"response\":{\"status\":\"incomplete\",\"incomplete_details\":{\"reason\":\"max_output_tokens\"},\"usage\":{\"input_tokens\":10,\"output_tokens\":3}}}\n\n"
        );

        let merged = convert_stream_text(input).await;
        assert!(merged.contains("\"stop_reason\":\"max_tokens\""));
        assert!(merged.contains("event: message_stop"));
        assert!(!merged.contains("event: error"));
    }

    #[tokio::test]
    async fn test_response_incomplete_event_without_status_uses_event_fallback() {
        let input = concat!(
            "event: response.incomplete\n",
            "data: {\"type\":\"response.incomplete\",\"response\":{\"usage\":{\"output_tokens\":3}}}\n\n"
        );

        let merged = convert_stream_text(input).await;
        assert!(merged.contains("\"stop_reason\":\"max_tokens\""));
        assert!(merged.contains("event: message_stop"));
    }

    #[tokio::test]
    async fn test_response_incomplete_zero_usage_empty_output_is_upstream_error() {
        // Shape returned by the ChatGPT Codex backend when it rejects the
        // request before processing it (e.g. over a tool schema pattern):
        // incomplete/max_output_tokens with zero usage and an empty output.
        let input = concat!(
            "event: response.created\n",
            "data: {\"type\":\"response.created\",\"response\":{\"id\":\"resp_1\",\"model\":\"gpt-5\"}}\n\n",
            "event: response.incomplete\n",
            "data: {\"type\":\"response.incomplete\",\"response\":{\"id\":\"resp_1\",\"status\":\"incomplete\",\"incomplete_details\":{\"reason\":\"max_output_tokens\"},\"usage\":{\"input_tokens\":0,\"output_tokens\":0},\"output\":[]}}\n\n"
        );

        let merged = convert_stream_text(input).await;
        assert!(merged.contains("event: message_start"));
        assert!(merged.contains("event: error"));
        assert!(merged.contains("Upstream rejected the request before processing"));
        assert!(!merged.contains("\"stop_reason\":\"max_tokens\""));
        assert!(!merged.contains("event: message_stop"));
    }

    #[tokio::test]
    async fn test_response_incomplete_zero_usage_without_output_array_is_upstream_error() {
        // Same rejection without an `output` array, which skips the
        // empty-output fast path and takes the regular terminal handling.
        let input = concat!(
            "event: response.created\n",
            "data: {\"type\":\"response.created\",\"response\":{\"id\":\"resp_1\",\"model\":\"gpt-5\"}}\n\n",
            "event: response.incomplete\n",
            "data: {\"type\":\"response.incomplete\",\"response\":{\"id\":\"resp_1\",\"status\":\"incomplete\",\"incomplete_details\":{\"reason\":\"max_output_tokens\"},\"usage\":{\"input_tokens\":0,\"output_tokens\":0}}}\n\n"
        );

        let merged = convert_stream_text(input).await;
        assert!(merged.contains("event: error"));
        assert!(merged.contains("Upstream rejected the request before processing"));
        assert!(!merged.contains("\"stop_reason\":\"max_tokens\""));
    }

    #[tokio::test]
    async fn test_response_incomplete_zero_input_nonzero_output_keeps_max_tokens() {
        // Non-zero output usage means tokens were actually generated: keep
        // reporting it as truncation.
        let input = concat!(
            "event: response.created\n",
            "data: {\"type\":\"response.created\",\"response\":{\"id\":\"resp_1\",\"model\":\"gpt-5\"}}\n\n",
            "event: response.incomplete\n",
            "data: {\"type\":\"response.incomplete\",\"response\":{\"id\":\"resp_1\",\"status\":\"incomplete\",\"incomplete_details\":{\"reason\":\"max_output_tokens\"},\"usage\":{\"input_tokens\":0,\"output_tokens\":7},\"output\":[]}}\n\n"
        );

        let merged = convert_stream_text(input).await;
        assert!(merged.contains("\"stop_reason\":\"max_tokens\""));
        assert!(merged.contains("event: message_stop"));
        assert!(!merged.contains("event: error"));
    }

    #[test]
    fn test_responses_json_to_anthropic_sse_zero_usage_incomplete_is_error() {
        // Also covers the compatible-gateway path that feeds a full Responses
        // JSON document straight into the converter.
        let events = responses_json_to_anthropic_sse(
            json!({
                "id": "resp_1",
                "status": "incomplete",
                "incomplete_details": {"reason": "max_output_tokens"},
                "usage": {"input_tokens": 0, "output_tokens": 0},
                "output": []
            }),
            None,
            None,
        );
        let merged = events
            .iter()
            .map(|event| String::from_utf8_lossy(event).to_string())
            .collect::<String>();
        assert!(merged.contains("event: message_start"));
        assert!(merged.contains("event: error"));
        assert!(merged.contains("Upstream rejected the request before processing"));
        assert!(!merged.contains("\"stop_reason\":\"max_tokens\""));
    }


}
