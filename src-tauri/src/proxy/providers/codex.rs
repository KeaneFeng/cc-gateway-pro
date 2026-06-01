//! Codex (OpenAI) Provider Adapter
//!
//! 支持两种 API 格式：
//! - **openai_responses** (默认): OpenAI Responses API (`/v1/responses`)
//! - **openai_chat**: OpenAI Chat Completions API (`/v1/chat/completions`)
//!
//! ## 客户端检测
//! 支持检测官方 Codex 客户端 (codex_vscode, codex_cli_rs)

use super::{AuthInfo, AuthStrategy, ProviderAdapter};
use crate::provider::Provider;
use crate::proxy::error::ProxyError;
use regex::Regex;
use serde_json::{json, Value};
use std::sync::LazyLock;

/// 官方 Codex 客户端 User-Agent 正则
#[allow(dead_code)]
static CODEX_CLIENT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(codex_vscode|codex_cli_rs)/[\d.]+").unwrap());

/// Codex 适配器
pub struct CodexAdapter;

impl CodexAdapter {
    pub fn new() -> Self {
        Self
    }

    /// 检测是否为官方 Codex 客户端
    ///
    /// 匹配 User-Agent 模式: `^(codex_vscode|codex_cli_rs)/[\d.]+`
    #[allow(dead_code)]
    pub fn is_official_client(user_agent: &str) -> bool {
        CODEX_CLIENT_REGEX.is_match(user_agent)
    }

    /// 从 Provider 配置中提取 API Key
    fn extract_key(&self, provider: &Provider) -> Option<String> {
        // 1. 尝试从 env 中获取
        if let Some(env) = provider.settings_config.get("env") {
            if let Some(key) = env.get("OPENAI_API_KEY").and_then(|v| v.as_str()) {
                return Some(key.to_string());
            }
        }

        // 2. 尝试从 auth 中获取 (Codex CLI 格式)
        if let Some(auth) = provider.settings_config.get("auth") {
            if let Some(key) = auth.get("OPENAI_API_KEY").and_then(|v| v.as_str()) {
                return Some(key.to_string());
            }
        }

        // 3. 尝试直接获取
        if let Some(key) = provider
            .settings_config
            .get("apiKey")
            .or_else(|| provider.settings_config.get("api_key"))
            .and_then(|v| v.as_str())
        {
            return Some(key.to_string());
        }

        // 4. 尝试从 config 对象中获取
        if let Some(config) = provider.settings_config.get("config") {
            if let Some(key) = config
                .get("api_key")
                .or_else(|| config.get("apiKey"))
                .and_then(|v| v.as_str())
            {
                return Some(key.to_string());
            }
        }

        None
    }
}

impl Default for CodexAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// 获取 Codex provider 的 API 格式
///
/// 优先级：meta.apiFormat > settings_config.api_format > 默认 "openai_responses"
pub fn get_codex_api_format(provider: &Provider) -> &'static str {
    // 1) Preferred: meta.apiFormat
    if let Some(meta) = provider.meta.as_ref() {
        if let Some(api_format) = meta.api_format.as_deref() {
            return match api_format {
                "openai_chat" => "openai_chat",
                "openai_responses" => "openai_responses",
                _ => "openai_responses",
            };
        }
    }

    // 2) Backward compatibility: settings_config.api_format
    if let Some(api_format) = provider
        .settings_config
        .get("api_format")
        .and_then(|v| v.as_str())
    {
        return match api_format {
            "openai_chat" => "openai_chat",
            "openai_responses" => "openai_responses",
            _ => "openai_responses",
        };
    }

    // 3) 默认 Responses API
    "openai_responses"
}

/// 检查 Codex API 格式是否需要转换
pub fn codex_api_format_needs_transform(api_format: &str) -> bool {
    api_format == "openai_chat"
}

/// Responses API → Chat Completions API 转换
///
/// 主要差异：
/// - `input` → `messages`
/// - `instructions` → system role message
/// - `max_output_tokens` → `max_tokens`
#[allow(dead_code)]
pub fn responses_to_chat_completions(body: Value) -> Result<Value, ProxyError> {
    let mut result = json!({});

    // 模型直接透传
    if let Some(model) = body.get("model") {
        result["model"] = model.clone();
    }

    // instructions → system role message
    let mut messages = Vec::new();
    if let Some(instructions) = body.get("instructions") {
        let system_text = if let Some(s) = instructions.as_str() {
            s.to_string()
        } else if let Some(arr) = instructions.as_array() {
            arr.iter()
                .filter_map(|item| {
                    item.get("text")
                        .and_then(|t| t.as_str())
                        .map(|s| s.to_string())
                })
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            String::new()
        };
        if !system_text.is_empty() {
            messages.push(json!({
                "role": "system",
                "content": system_text
            }));
        }
    }

    // input → messages
    if let Some(input) = body.get("input") {
        if let Some(arr) = input.as_array() {
            for item in arr {
                if item.get("role").is_some() {
                    // 已经是 message 格式
                    messages.push(item.clone());
                } else if let Some(type_val) = item.get("type") {
                    // Responses API 的 input item 格式
                    match type_val.as_str() {
                        Some("message") => {
                            if let Some(content) = item.get("content") {
                                let role =
                                    item.get("role").and_then(|r| r.as_str()).unwrap_or("user");
                                messages.push(json!({
                                    "role": role,
                                    "content": content
                                }));
                            }
                        }
                        _ => {
                            // 其他类型（如 function_call_output）直接保留
                            messages.push(item.clone());
                        }
                    }
                }
            }
        } else if let Some(s) = input.as_str() {
            // 简单字符串 input
            messages.push(json!({
                "role": "user",
                "content": s
            }));
        }
    }

    if !messages.is_empty() {
        result["messages"] = json!(messages);
    }

    // max_output_tokens → max_tokens
    if let Some(v) = body.get("max_output_tokens") {
        result["max_tokens"] = v.clone();
    } else if let Some(v) = body.get("max_tokens") {
        result["max_tokens"] = v.clone();
    }

    // 转换 tools 格式（Responses API → Chat Completions API）
    // Responses API: {"type": "function", "name": "...", "parameters": {...}}
    // Chat Completions: {"type": "function", "function": {"name": "...", "parameters": {...}}}
    if let Some(tools) = body.get("tools").and_then(|t| t.as_array()) {
        let converted_tools: Vec<Value> = tools
            .iter()
            .map(|tool| {
                if let Some(name) = tool.get("name").and_then(|n| n.as_str()) {
                    // Responses API 格式，需要转换
                    json!({
                        "type": "function",
                        "function": {
                            "name": name,
                            "description": tool.get("description"),
                            "parameters": tool.get("parameters")
                        }
                    })
                } else {
                    // 已经是标准格式，直接透传
                    tool.clone()
                }
            })
            .collect();
        result["tools"] = json!(converted_tools);
    }

    // 直接透传的参数
    for key in &[
        "temperature",
        "top_p",
        "stream",
        "tool_choice",
        "response_format",
        "seed",
        "user",
        "n",
        "logprobs",
        "top_logprobs",
        "presence_penalty",
        "frequency_penalty",
        "logit_bias",
        "stop",
    ] {
        if let Some(v) = body.get(*key) {
            result[*key] = v.clone();
        }
    }

    // 透传 _ 前缀的私有字段
    for (key, value) in body.as_object().unwrap_or(&serde_json::Map::new()) {
        if key.starts_with('_') {
            result[key] = value.clone();
        }
    }

    Ok(result)
}

/// Codex CLI 默认模型列表（用于 model availability check）
const CODEX_DEFAULT_MODELS: &[&str] = &["gpt-5.4", "gpt-5.5", "gpt-4o", "gpt-4.1"];

/// Codex 模型映射：把 Codex 默认模型映射到 provider 的实际模型
///
/// Codex CLI 会用默认模型（如 gpt-5.4）做 model availability check，
/// 然后才切换到配置的模型。对于非 OpenAI 的 provider，需要把默认模型
/// 映射到 provider 支持的模型。
fn apply_codex_model_mapping(mut body: Value, provider: &Provider) -> Value {
    let model = body.get("model").and_then(|m| m.as_str()).unwrap_or("");
    let is_codex_default = CODEX_DEFAULT_MODELS.contains(&model);

    if !is_codex_default {
        return body;
    }

    // 从 provider 配置中获取实际模型名
    let actual_model = extract_codex_model_from_config(provider);
    if let Some(actual) = actual_model {
        if actual != model {
            log::debug!(
                "[Codex] 模型映射: {} -> {} (provider: {})",
                model,
                actual,
                provider.name
            );
            body["model"] = serde_json::json!(actual);
        }
    }

    body
}

/// 从 Codex provider 配置中提取模型名
fn extract_codex_model_from_config(provider: &Provider) -> Option<String> {
    // 从 config 字符串中解析 model
    if let Some(config) = provider.settings_config.get("config") {
        if let Some(config_str) = config.as_str() {
            // 解析 TOML 格式的 model = "xxx"
            for line in config_str.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("model")
                    && trimmed.contains('=')
                    && !trimmed.starts_with("model_")
                {
                    if let Some(value) = trimmed.split('=').nth(1) {
                        let value = value.trim().trim_matches('"');
                        if !value.is_empty() {
                            return Some(value.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

impl ProviderAdapter for CodexAdapter {
    fn name(&self) -> &'static str {
        "Codex"
    }

    fn extract_base_url(&self, provider: &Provider) -> Result<String, ProxyError> {
        // 1. 尝试直接获取 base_url 字段
        if let Some(url) = provider
            .settings_config
            .get("base_url")
            .and_then(|v| v.as_str())
        {
            return Ok(url.trim_end_matches('/').to_string());
        }

        // 2. 尝试 baseURL
        if let Some(url) = provider
            .settings_config
            .get("baseURL")
            .and_then(|v| v.as_str())
        {
            return Ok(url.trim_end_matches('/').to_string());
        }

        // 3. 尝试从 config 对象中获取
        if let Some(config) = provider.settings_config.get("config") {
            if let Some(url) = config.get("base_url").and_then(|v| v.as_str()) {
                return Ok(url.trim_end_matches('/').to_string());
            }

            // 尝试解析 TOML 字符串格式
            if let Some(config_str) = config.as_str() {
                if let Some(start) = config_str.find("base_url = \"") {
                    let rest = &config_str[start + 12..];
                    if let Some(end) = rest.find('"') {
                        return Ok(rest[..end].trim_end_matches('/').to_string());
                    }
                }
                if let Some(start) = config_str.find("base_url = '") {
                    let rest = &config_str[start + 12..];
                    if let Some(end) = rest.find('\'') {
                        return Ok(rest[..end].trim_end_matches('/').to_string());
                    }
                }
            }
        }

        Err(ProxyError::ConfigError(
            "Codex Provider 缺少 base_url 配置".to_string(),
        ))
    }

    fn extract_auth(&self, provider: &Provider) -> Option<AuthInfo> {
        self.extract_key(provider)
            .map(|key| AuthInfo::new(key, AuthStrategy::Bearer))
    }

    fn build_url(&self, base_url: &str, endpoint: &str) -> String {
        let base_trimmed = base_url.trim_end_matches('/');
        let endpoint_trimmed = endpoint.trim_start_matches('/');

        // OpenAI/Codex 的 base_url 可能是：
        // - 纯 origin: https://api.openai.com  (需要自动补 /v1)
        // - 已含 /v1: https://api.openai.com/v1 (直接拼接)
        // - 自定义前缀: https://xxx/openai (不添加 /v1，直接拼接)

        // 检查 base_url 是否已经包含 /v1
        let already_has_v1 = base_trimmed.ends_with("/v1");

        // 检查是否是纯 origin（没有路径部分）
        let origin_only = match base_trimmed.split_once("://") {
            Some((_scheme, rest)) => !rest.contains('/'),
            None => !base_trimmed.contains('/'),
        };

        let mut url = if already_has_v1 {
            // 已经有 /v1，直接拼接
            format!("{base_trimmed}/{endpoint_trimmed}")
        } else if origin_only {
            // 纯 origin，添加 /v1
            format!("{base_trimmed}/v1/{endpoint_trimmed}")
        } else {
            // 自定义前缀，不添加 /v1，直接拼接
            format!("{base_trimmed}/{endpoint_trimmed}")
        };

        // 去除重复的 /v1/v1（可能由 base_url 与 endpoint 都带版本导致）
        while url.contains("/v1/v1") {
            url = url.replace("/v1/v1", "/v1");
        }

        url
    }

    fn get_auth_headers(
        &self,
        auth: &AuthInfo,
    ) -> Result<Vec<(http::HeaderName, http::HeaderValue)>, ProxyError> {
        use super::adapter::auth_header_value;
        let bearer = format!("Bearer {}", auth.api_key);
        Ok(vec![(
            http::HeaderName::from_static("authorization"),
            auth_header_value(&bearer)?,
        )])
    }

    fn needs_transform(&self, provider: &Provider) -> bool {
        let api_format = get_codex_api_format(provider);
        codex_api_format_needs_transform(api_format)
    }

    fn transform_request(&self, body: Value, provider: &Provider) -> Result<Value, ProxyError> {
        let api_format = get_codex_api_format(provider);
        let body = match api_format {
            "openai_chat" => super::transform_codex_chat::responses_to_chat_completions(body)?,
            _ => body, // openai_responses 不需要转换
        };

        // Codex 模型映射：把 Codex 默认模型（如 gpt-5.4）映射到 provider 的实际模型
        // 解决 Codex CLI 用默认模型做 model availability check 的问题
        let mapped_body = apply_codex_model_mapping(body, provider);

        Ok(mapped_body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_provider(config: serde_json::Value) -> Provider {
        Provider {
            id: "test".to_string(),
            name: "Test Codex".to_string(),
            settings_config: config,
            website_url: None,
            category: Some("codex".to_string()),
            created_at: None,
            sort_index: None,
            notes: None,
            meta: None,
            icon: None,
            icon_color: None,
            in_failover_queue: false,
        }
    }

    fn create_provider_with_meta(config: serde_json::Value, api_format: Option<&str>) -> Provider {
        use crate::provider::ProviderMeta;
        Provider {
            id: "test".to_string(),
            name: "Test Codex".to_string(),
            settings_config: config,
            website_url: None,
            category: Some("codex".to_string()),
            created_at: None,
            sort_index: None,
            notes: None,
            meta: Some(ProviderMeta {
                api_format: api_format.map(|s| s.to_string()),
                ..Default::default()
            }),
            icon: None,
            icon_color: None,
            in_failover_queue: false,
        }
    }

    #[test]
    fn test_extract_base_url_direct() {
        let adapter = CodexAdapter::new();
        let provider = create_provider(json!({
            "base_url": "https://api.openai.com/v1"
        }));

        let url = adapter.extract_base_url(&provider).unwrap();
        assert_eq!(url, "https://api.openai.com/v1");
    }

    #[test]
    fn test_extract_auth_from_auth_field() {
        let adapter = CodexAdapter::new();
        let provider = create_provider(json!({
            "auth": {
                "OPENAI_API_KEY": "sk-tes...5678"
            }
        }));

        let auth = adapter.extract_auth(&provider).unwrap();
        assert_eq!(auth.api_key, "sk-tes...5678");
        assert_eq!(auth.strategy, AuthStrategy::Bearer);
    }

    #[test]
    fn test_extract_auth_from_env() {
        let adapter = CodexAdapter::new();
        let provider = create_provider(json!({
            "env": {
                "OPENAI_API_KEY": "sk-env...5678"
            }
        }));

        let auth = adapter.extract_auth(&provider).unwrap();
        assert_eq!(auth.api_key, "sk-env...5678");
    }

    #[test]
    fn test_build_url() {
        let adapter = CodexAdapter::new();
        let url = adapter.build_url("https://api.openai.com/v1", "/responses");
        assert_eq!(url, "https://api.openai.com/v1/responses");
    }

    #[test]
    fn test_build_url_origin_adds_v1() {
        let adapter = CodexAdapter::new();
        let url = adapter.build_url("https://api.openai.com", "/responses");
        assert_eq!(url, "https://api.openai.com/v1/responses");
    }

    #[test]
    fn test_build_url_custom_prefix_no_v1() {
        let adapter = CodexAdapter::new();
        let url = adapter.build_url("https://example.com/openai", "/responses");
        assert_eq!(url, "https://example.com/openai/responses");
    }

    #[test]
    fn test_build_url_dedup_v1() {
        let adapter = CodexAdapter::new();
        // base_url 已包含 /v1，endpoint 也包含 /v1
        let url = adapter.build_url("https://www.packyapi.com/v1", "/v1/responses");
        assert_eq!(url, "https://www.packyapi.com/v1/responses");
    }

    // 官方客户端检测测试
    #[test]
    fn test_is_official_client_vscode() {
        assert!(CodexAdapter::is_official_client("codex_vscode/1.0.0"));
        assert!(CodexAdapter::is_official_client("codex_vscode/2.3.4"));
        assert!(CodexAdapter::is_official_client("codex_vscode/0.1"));
    }

    #[test]
    fn test_is_official_client_cli() {
        assert!(CodexAdapter::is_official_client("codex_cli_rs/1.0.0"));
        assert!(CodexAdapter::is_official_client("codex_cli_rs/0.5.2"));
    }

    #[test]
    fn test_is_not_official_client() {
        assert!(!CodexAdapter::is_official_client("Mozilla/5.0"));
        assert!(!CodexAdapter::is_official_client("curl/7.68.0"));
        assert!(!CodexAdapter::is_official_client("python-requests/2.25.1"));
        assert!(!CodexAdapter::is_official_client("codex_other/1.0.0"));
        assert!(!CodexAdapter::is_official_client(""));
    }

    #[test]
    fn test_is_official_client_partial_match() {
        // 必须从开头匹配
        assert!(!CodexAdapter::is_official_client("some codex_vscode/1.0.0"));
        assert!(!CodexAdapter::is_official_client(
            "prefix_codex_cli_rs/1.0.0"
        ));
    }

    // API 格式测试
    #[test]
    fn test_get_codex_api_format_default() {
        let provider = create_provider(json!({}));
        assert_eq!(get_codex_api_format(&provider), "openai_responses");
    }

    #[test]
    fn test_get_codex_api_format_from_config() {
        let provider = create_provider(json!({
            "api_format": "openai_chat"
        }));
        assert_eq!(get_codex_api_format(&provider), "openai_chat");
    }

    #[test]
    fn test_get_codex_api_format_from_meta() {
        let provider = create_provider_with_meta(json!({}), Some("openai_chat"));
        assert_eq!(get_codex_api_format(&provider), "openai_chat");
    }

    #[test]
    fn test_codex_api_format_needs_transform() {
        assert!(codex_api_format_needs_transform("openai_chat"));
        assert!(!codex_api_format_needs_transform("openai_responses"));
    }

    #[test]
    fn test_needs_transform_default() {
        let adapter = CodexAdapter::new();
        let provider = create_provider(json!({}));
        assert!(!adapter.needs_transform(&provider)); // 默认不需要转换
    }

    #[test]
    fn test_needs_transform_openai_chat() {
        let adapter = CodexAdapter::new();
        let provider = create_provider(json!({
            "api_format": "openai_chat"
        }));
        assert!(adapter.needs_transform(&provider));
    }

    // Responses → Chat Completions 转换测试
    #[test]
    fn test_responses_to_chat_completions_basic() {
        let input = json!({
            "model": "mimo-v2.5-pro",
            "input": [
                {"role": "user", "content": "Hello"}
            ],
            "instructions": "You are a helpful assistant",
            "max_output_tokens": 1000,
            "stream": true
        });

        let result = responses_to_chat_completions(input).unwrap();

        assert_eq!(result["model"], "mimo-v2.5-pro");
        assert_eq!(result["stream"], true);
        assert_eq!(result["max_tokens"], 1000);

        let messages = result["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0]["role"], "system");
        assert_eq!(messages[0]["content"], "You are a helpful assistant");
        assert_eq!(messages[1]["role"], "user");
        assert_eq!(messages[1]["content"], "Hello");
    }

    #[test]
    fn test_responses_to_chat_completions_no_instructions() {
        let input = json!({
            "model": "mimo-v2.5-pro",
            "input": [
                {"role": "user", "content": "Hello"}
            ],
            "max_output_tokens": 500
        });

        let result = responses_to_chat_completions(input).unwrap();

        let messages = result["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[0]["content"], "Hello");
        assert_eq!(result["max_tokens"], 500);
    }

    #[test]
    fn test_responses_to_chat_completions_string_input() {
        let input = json!({
            "model": "mimo-v2.5-pro",
            "input": "Hello world"
        });

        let result = responses_to_chat_completions(input).unwrap();

        let messages = result["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[0]["content"], "Hello world");
    }

    #[test]
    fn test_responses_to_chat_completions_preserves_tools() {
        let input = json!({
            "model": "mimo-v2.5-pro",
            "input": [{"role": "user", "content": "test"}],
            "tools": [{"type": "function", "function": {"name": "test"}}],
            "tool_choice": "auto"
        });

        let result = responses_to_chat_completions(input).unwrap();

        assert!(result["tools"].is_array());
        assert_eq!(result["tool_choice"], "auto");
    }
}
