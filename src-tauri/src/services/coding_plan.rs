//! 国产 Token Plan 额度查询服务
//!
//! 支持 Kimi For Coding、智谱 GLM、MiniMax 的 Token Plan 额度查询。
//! 复用 subscription 模块的 SubscriptionQuota / QuotaTier 类型。

use super::subscription::{
    CredentialStatus, QuotaTier, SubscriptionQuota, TIER_FIVE_HOUR, TIER_WEEKLY_LIMIT,
};
use std::time::{SystemTime, UNIX_EPOCH};

// ── 供应商检测 ──────────────────────────────────────────────

enum CodingPlanProvider {
    Kimi,
    ZhipuCn,
    ZhipuEn,
    MiniMaxCn,
    MiniMaxEn,
}

fn detect_provider(base_url: &str) -> Option<CodingPlanProvider> {
    let url = base_url.to_lowercase();
    if url.contains("api.kimi.com/coding") {
        Some(CodingPlanProvider::Kimi)
    } else if url.contains("open.bigmodel.cn") || url.contains("bigmodel.cn") {
        Some(CodingPlanProvider::ZhipuCn)
    } else if url.contains("api.z.ai") {
        Some(CodingPlanProvider::ZhipuEn)
    } else if url.contains("api.minimaxi.com") {
        Some(CodingPlanProvider::MiniMaxCn)
    } else if url.contains("api.minimax.io") {
        Some(CodingPlanProvider::MiniMaxEn)
    } else {
        None
    }
}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn millis_to_iso8601(ms: i64) -> Option<String> {
    let secs = ms / 1000;
    let nsecs = ((ms % 1000) * 1_000_000) as u32;
    chrono::DateTime::from_timestamp(secs, nsecs).map(|dt| dt.to_rfc3339())
}

/// 从 JSON 值提取重置时间，兼容字符串和数字格式
/// - 字符串：直接返回（ISO 8601）
/// - 数字：自动判断秒/毫秒并转为 ISO 8601
fn extract_reset_time(value: &serde_json::Value) -> Option<String> {
    if let Some(s) = value.as_str() {
        return Some(s.to_string());
    }
    if let Some(n) = value.as_i64() {
        // 区分秒和毫秒：秒级时间戳 < 1e12，毫秒 >= 1e12
        let ms = if n < 1_000_000_000_000 { n * 1000 } else { n };
        return millis_to_iso8601(ms);
    }
    None
}

/// 解析 JSON 值为 f64，兼容数字和字符串格式（如 `100` 和 `"100"`）
fn parse_f64(value: &serde_json::Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_str().and_then(|s| s.parse().ok()))
}

fn make_error(msg: String) -> SubscriptionQuota {
    SubscriptionQuota {
        tool: "coding_plan".to_string(),
        credential_status: CredentialStatus::Valid,
        credential_message: None,
        success: false,
        tiers: vec![],
        extra_usage: None,
        error: Some(msg),
        queried_at: Some(now_millis()),
    }
}

// ── Kimi For Coding ─────────────────────────────────────────

async fn query_kimi(api_key: &str) -> SubscriptionQuota {
    let client = crate::proxy::http_client::get();

    let resp = client
        .get("https://api.kimi.com/coding/v1/usages")
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Accept", "application/json")
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await;

    let resp = match resp {
        Ok(r) => r,
        Err(e) => return make_error(format!("Network error: {e}")),
    };

    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        return SubscriptionQuota {
            tool: "coding_plan".to_string(),
            credential_status: CredentialStatus::Expired,
            credential_message: Some("Invalid API key".to_string()),
            success: false,
            tiers: vec![],
            extra_usage: None,
            error: Some(format!("Authentication failed (HTTP {status})")),
            queried_at: Some(now_millis()),
        };
    }

    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return make_error(format!("API error (HTTP {status}): {body}"));
    }

    let body: serde_json::Value = match resp.json().await {
        Ok(v) => v,
        Err(e) => return make_error(format!("Failed to parse response: {e}")),
    };

    let mut tiers = Vec::new();

    // 5 小时窗口限额（优先显示）
    if let Some(limits) = body.get("limits").and_then(|v| v.as_array()) {
        for limit_item in limits {
            if let Some(detail) = limit_item.get("detail") {
                let limit = detail.get("limit").and_then(parse_f64).unwrap_or(1.0);
                let remaining = detail.get("remaining").and_then(parse_f64).unwrap_or(0.0);
                let resets_at = detail.get("resetTime").and_then(extract_reset_time);

                let used = (limit - remaining).max(0.0);
                let utilization = if limit > 0.0 {
                    (used / limit) * 100.0
                } else {
                    0.0
                };
                tiers.push(QuotaTier {
                    name: "five_hour".to_string(),
                    utilization,
                    resets_at,
                });
            }
        }
    }

    // 总体用量（周限额）
    if let Some(usage) = body.get("usage") {
        let limit = usage.get("limit").and_then(parse_f64).unwrap_or(1.0);
        let remaining = usage.get("remaining").and_then(parse_f64).unwrap_or(0.0);
        let resets_at = usage.get("resetTime").and_then(extract_reset_time);

        let used = (limit - remaining).max(0.0);
        let utilization = if limit > 0.0 {
            (used / limit) * 100.0
        } else {
            0.0
        };
        tiers.push(QuotaTier {
            name: "weekly_limit".to_string(),
            utilization,
            resets_at,
        });
    }

    SubscriptionQuota {
        tool: "coding_plan".to_string(),
        credential_status: CredentialStatus::Valid,
        credential_message: None,
        success: true,
        tiers,
        extra_usage: None,
        error: None,
        queried_at: Some(now_millis()),
    }
}

// ── 智谱 GLM ────────────────────────────────────────────────

// 把智谱 `data` 里的 `limits[]` 解析成 tier 列表。
//
// 双桶响应中，5 小时桶在 0% 等状态下可能没有 `nextResetTime`；
// 这类无 reset 条目应优先归为五小时桶。其余条目按 `nextResetTime` 升序。
// ── 智谱 GLM ────────────────────────────────────────────────

/// 智谱 TOKENS_LIMIT 条目按 `unit` 字段的显式窗口分类。
enum ZhipuWindow {
    FiveHour,
    Weekly,
}

/// 按 `unit` 字段判定 TOKENS_LIMIT 条目所属窗口。
///
/// 实测形态（bigmodel.cn 与 z.ai 共用同一后端，字段一致）：
/// - `unit: 3, number: 5` → 5 小时滚动窗口（老/新套餐均有）
/// - `unit: 6, number: 7` 与 `unit: 6, number: 1` → 每周窗口（两种取值都被
///   实测过，故只锚定 `unit`、不绑 `number`）
///
/// `unit` 缺失或值不认识时返回 None，由调用方走重置时间启发式兜底。
fn classify_zhipu_window(item: &serde_json::Value) -> Option<ZhipuWindow> {
    match item.get("unit").and_then(|v| v.as_i64()) {
        Some(3) => Some(ZhipuWindow::FiveHour),
        Some(6) => Some(ZhipuWindow::Weekly),
        _ => None,
    }
}

/// 把智谱 `data` 里的 `limits[]` 解析成 tier 列表。
///
/// 分类优先级：
/// 1. 显式字段：`unit` 标识窗口类型（见 [`classify_zhipu_window`]）。不能按
///    `nextResetTime` 排序代替——周期末尾每周窗口会比 5 小时窗口更早重置
///    （issue #3036），时间排序在该场景必然把两桶标反。
/// 2. 兜底启发式（`unit` 缺失或不识别）：无 `nextResetTime` 的条目优先归
///    five_hour（5 小时桶在 0% 等状态下可能没有 reset），其余按 reset 升序
///    依次填入仍空缺的槽位。
///
/// 老套餐（2026-02-12 前订阅）只回 1 条
/// `TOKENS_LIMIT`，自然降级为仅展示 `five_hour`；新套餐回 2 条。
fn parse_zhipu_token_tiers(data: &serde_json::Value) -> Vec<QuotaTier> {
    type Entry = (Option<i64>, f64, Option<String>);
    let mut five_hour: Option<Entry> = None;
    let mut weekly: Option<Entry> = None;
    let mut unclassified: Vec<Entry> = Vec::new();

    if let Some(limits) = data.get("limits").and_then(|v| v.as_array()) {
        for limit_item in limits {
            let limit_type = limit_item
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            // 大小写不敏感比较：上游若把 "TOKENS_LIMIT" 改成小写或驼峰，依然能识别
            if !limit_type.eq_ignore_ascii_case("TOKENS_LIMIT") {
                continue;
            }
            let percentage = limit_item
                .get("percentage")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let reset_ms = limit_item.get("nextResetTime").and_then(|v| v.as_i64());
            let reset_iso = reset_ms.and_then(millis_to_iso8601);
            let entry = (reset_ms, percentage, reset_iso);
            match classify_zhipu_window(limit_item) {
                Some(ZhipuWindow::FiveHour) if five_hour.is_none() => five_hour = Some(entry),
                Some(ZhipuWindow::Weekly) if weekly.is_none() => weekly = Some(entry),
                _ => unclassified.push(entry),
            }
        }
    }

    unclassified.sort_by_key(|(reset, _, _)| (reset.is_some(), reset.unwrap_or(i64::MIN)));
    for entry in unclassified {
        if five_hour.is_none() {
            five_hour = Some(entry);
        } else if weekly.is_none() {
            weekly = Some(entry);
        }
        // 智谱当前最多两条 TOKENS_LIMIT，多余的忽略
    }

    let mut tiers = Vec::new();
    for (name, slot) in [(TIER_FIVE_HOUR, five_hour), (TIER_WEEKLY_LIMIT, weekly)] {
        if let Some((_, percentage, resets_at)) = slot {
            tiers.push(QuotaTier {
                name: name.to_string(),
                utilization: percentage,
                resets_at,
            });
        }
    }
    tiers
}

/// Resolve the Zhipu quota endpoint from the user's configured `base_url`.
///
/// Zhipu ships as two distinct presets (Zhipu GLM = `open.bigmodel.cn`,
/// Zhipu GLM en = `api.z.ai`) that share the same quota path and JSON shape.
/// The quota endpoint lives on the same host as the user's coding endpoint,
/// so we route by `base_url` and let the caller's existing reachability
/// (they're already using this host to run coding) determine success — no
/// cross-host fallback, no auth-error heuristics.
fn zhipu_quota_base(base_url: &str) -> &'static str {
    if base_url.to_lowercase().contains("bigmodel.cn") {
        "https://open.bigmodel.cn"
    } else {
        "https://api.z.ai"
    }
}

async fn query_zhipu(base_url: &str, api_key: &str) -> SubscriptionQuota {
    let client = crate::proxy::http_client::get();
    let url = format!(
        "{}/api/monitor/usage/quota/limit",
        zhipu_quota_base(base_url)
    );

    let resp = client
        .get(&url)
        .header("Authorization", api_key) // 注意：智谱不加 Bearer 前缀
        .header("Content-Type", "application/json")
        .header("Accept-Language", "en-US,en")
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await;

    let resp = match resp {
        Ok(r) => r,
        Err(e) => return make_error(format!("Network error: {e}")),
    };

    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        return SubscriptionQuota {
            tool: "coding_plan".to_string(),
            credential_status: CredentialStatus::Expired,
            credential_message: Some("Invalid API key".to_string()),
            success: false,
            tiers: vec![],
            extra_usage: None,
            error: Some(format!("Authentication failed (HTTP {status})")),
            queried_at: Some(now_millis()),
        };
    }

    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return make_error(format!("API error (HTTP {status}): {body}"));
    }

    let body: serde_json::Value = match resp.json().await {
        Ok(v) => v,
        Err(e) => return make_error(format!("Failed to parse response: {e}")),
    };

    // 检查业务级别错误
    if body.get("success").and_then(|v| v.as_bool()) == Some(false) {
        let msg = body
            .get("msg")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error");
        return make_error(format!("API error: {msg}"));
    }

    let data = match body.get("data") {
        Some(d) => d,
        None => return make_error("Missing 'data' field in response".to_string()),
    };

    let tiers = parse_zhipu_token_tiers(data);

    // 套餐等级存入 credential_message
    let level = data
        .get("level")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    SubscriptionQuota {
        tool: "coding_plan".to_string(),
        credential_status: CredentialStatus::Valid,
        credential_message: level,
        success: true,
        tiers,
        extra_usage: None,
        error: None,
        queried_at: Some(now_millis()),
    }
}

// ── MiniMax ─────────────────────────────────────────────────

async fn query_minimax(api_key: &str, is_cn: bool) -> SubscriptionQuota {
    let client = crate::proxy::http_client::get();

    let api_domain = if is_cn {
        "api.minimaxi.com"
    } else {
        "api.minimax.io"
    };
    let url = format!("https://{api_domain}/v1/api/openplatform/coding_plan/remains");

    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await;

    let resp = match resp {
        Ok(r) => r,
        Err(e) => return make_error(format!("Network error: {e}")),
    };

    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        return SubscriptionQuota {
            tool: "coding_plan".to_string(),
            credential_status: CredentialStatus::Expired,
            credential_message: Some("Invalid API key".to_string()),
            success: false,
            tiers: vec![],
            extra_usage: None,
            error: Some(format!("Authentication failed (HTTP {status})")),
            queried_at: Some(now_millis()),
        };
    }

    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return make_error(format!("API error (HTTP {status}): {body}"));
    }

    let body: serde_json::Value = match resp.json().await {
        Ok(v) => v,
        Err(e) => return make_error(format!("Failed to parse response: {e}")),
    };

    // 检查业务级别错误
    if let Some(base_resp) = body.get("base_resp") {
        let status_code = base_resp
            .get("status_code")
            .and_then(|v| v.as_i64())
            .unwrap_or(-1);
        if status_code != 0 {
            let msg = base_resp
                .get("status_msg")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            return make_error(format!("API error (code {status_code}): {msg}"));
        }
    }

    let mut tiers = Vec::new();

    if let Some(model_remains) = body.get("model_remains").and_then(|v| v.as_array()) {
        // 只取第一个模型（MiniMax-M*，主力编程模型）
        if let Some(item) = model_remains.first() {
            // usage_count 是剩余量（满额=total，用完=0），需反转为已用百分比
            let interval_total = item
                .get("current_interval_total_count")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let interval_remaining = item
                .get("current_interval_usage_count")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let end_time = item.get("end_time").and_then(|v| v.as_i64());

            if interval_total > 0.0 {
                tiers.push(QuotaTier {
                    name: "five_hour".to_string(),
                    utilization: ((interval_total - interval_remaining) / interval_total) * 100.0,
                    resets_at: end_time.and_then(millis_to_iso8601),
                });
            }

            // 周额度
            let weekly_total = item
                .get("current_weekly_total_count")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let weekly_remaining = item
                .get("current_weekly_usage_count")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let weekly_end = item.get("weekly_end_time").and_then(|v| v.as_i64());

            if weekly_total > 0.0 {
                tiers.push(QuotaTier {
                    name: "weekly_limit".to_string(),
                    utilization: ((weekly_total - weekly_remaining) / weekly_total) * 100.0,
                    resets_at: weekly_end.and_then(millis_to_iso8601),
                });
            }
        }
    }

    SubscriptionQuota {
        tool: "coding_plan".to_string(),
        credential_status: CredentialStatus::Valid,
        credential_message: None,
        success: true,
        tiers,
        extra_usage: None,
        error: None,
        queried_at: Some(now_millis()),
    }
}

// ── 公开入口 ────────────────────────────────────────────────

pub async fn get_coding_plan_quota(
    base_url: &str,
    api_key: &str,
) -> Result<SubscriptionQuota, String> {
    if api_key.trim().is_empty() {
        return Ok(SubscriptionQuota {
            tool: "coding_plan".to_string(),
            credential_status: CredentialStatus::NotFound,
            credential_message: None,
            success: false,
            tiers: vec![],
            extra_usage: None,
            // 与 balance::get_balance 一致：给出明确错误，避免 footer 显示无信息的失败
            error: Some("API key is empty".to_string()),
            queried_at: None,
        });
    }

    let provider = match detect_provider(base_url) {
        Some(p) => p,
        None => {
            return Ok(SubscriptionQuota {
                tool: "coding_plan".to_string(),
                credential_status: CredentialStatus::NotFound,
                credential_message: None,
                success: false,
                tiers: vec![],
                extra_usage: None,
                // 域名未命中已知套餐供应商（如第三方中转站）：给出明确错误而非静默失败
                error: Some("Unknown coding plan provider".to_string()),
                queried_at: None,
            });
        }
    };

    let quota = match provider {
        CodingPlanProvider::Kimi => query_kimi(api_key).await,
        CodingPlanProvider::ZhipuCn | CodingPlanProvider::ZhipuEn => {
            query_zhipu(base_url, api_key).await
        }
        CodingPlanProvider::MiniMaxCn => query_minimax(api_key, true).await,
        CodingPlanProvider::MiniMaxEn => query_minimax(api_key, false).await,
    };

    Ok(quota)
}

#[cfg(test)]
mod tests {
    use super::{parse_zhipu_token_tiers, TIER_FIVE_HOUR, TIER_WEEKLY_LIMIT};
    use serde_json::json;

    #[test]
    fn zhipu_new_plan_two_tiers_sorted_by_reset_time() {
        // 新套餐：两条 TOKENS_LIMIT，nextResetTime 较近的归 five_hour、较远的归 weekly_limit。
        // 故意把"周限"放数组前面，验证不依赖输入顺序。
        let data = json!({
            "limits": [
                { "type": "TOKENS_LIMIT", "percentage": 53.0, "nextResetTime": 2_000_000_000_000_i64 },
                { "type": "TOKENS_LIMIT", "percentage": 44.0, "nextResetTime": 1_000_000_000_000_i64 },
                { "type": "TIME_LIMIT",   "percentage":  7.0 },
            ]
        });
        let tiers = parse_zhipu_token_tiers(&data);
        assert_eq!(tiers.len(), 2);
        assert_eq!(tiers[0].name, TIER_FIVE_HOUR);
        assert_eq!(tiers[0].utilization, 44.0);
        assert_eq!(tiers[1].name, TIER_WEEKLY_LIMIT);
        assert_eq!(tiers[1].utilization, 53.0);
    }

    #[test]
    fn zhipu_old_plan_single_tier_falls_back_to_five_hour() {
        // 老套餐（2026-02-12 前订阅）：仅一条 TOKENS_LIMIT，无周限。
        let data = json!({
            "limits": [
                {
                    "type": "TOKENS_LIMIT",
                    "percentage": 2.0,
                    "nextResetTime": 1_774_967_594_803_i64
                },
                { "type": "TIME_LIMIT", "percentage": 0.0 }
            ]
        });
        let tiers = parse_zhipu_token_tiers(&data);
        assert_eq!(tiers.len(), 1);
        assert_eq!(tiers[0].name, TIER_FIVE_HOUR);
        assert_eq!(tiers[0].utilization, 2.0);
    }

    #[test]
    fn zhipu_no_token_limits_returns_empty() {
        let data = json!({ "limits": [{ "type": "TIME_LIMIT", "percentage": 5.0 }] });
        assert!(parse_zhipu_token_tiers(&data).is_empty());
    }

    #[test]
    fn zhipu_missing_reset_time_is_five_hour_when_weekly_has_reset() {
        // 真实反馈：5 小时桶为 0% 时可能没有 nextResetTime；每周桶带 reset。
        // 这种形态不能按 reset 升序把每周桶误判为 five_hour。
        let data = json!({
            "limits": [
                { "type": "TOKENS_LIMIT", "percentage": 25.0, "nextResetTime": 2_000_000_000_000_i64 },
                { "type": "TOKENS_LIMIT", "percentage": 0.0 }
            ]
        });
        let tiers = parse_zhipu_token_tiers(&data);
        assert_eq!(tiers.len(), 2);
        assert_eq!(tiers[0].name, TIER_FIVE_HOUR);
        assert_eq!(tiers[0].utilization, 0.0);
        assert!(tiers[0].resets_at.is_none());
        assert_eq!(tiers[1].name, TIER_WEEKLY_LIMIT);
        assert_eq!(tiers[1].utilization, 25.0);
        assert!(tiers[1].resets_at.is_some());
    }

    #[test]
    fn zhipu_type_is_case_insensitive() {
        // 防御性：上游若把 "TOKENS_LIMIT" 改成 "tokens_limit"（仅大小写变化）仍能识别。
        // 注意：分隔符差异（如 "TokensLimit" 去掉下划线）不在兼容范围。
        let data = json!({
            "limits": [
                { "type": "tokens_limit", "percentage": 12.0, "nextResetTime": 1_000_000_000_000_i64 },
                { "type": "Tokens_Limit", "percentage": 34.0, "nextResetTime": 2_000_000_000_000_i64 }
            ]
        });
        let tiers = parse_zhipu_token_tiers(&data);
        assert_eq!(tiers.len(), 2);
        assert_eq!(tiers[0].name, TIER_FIVE_HOUR);
        assert_eq!(tiers[0].utilization, 12.0);
        assert_eq!(tiers[1].name, TIER_WEEKLY_LIMIT);
        assert_eq!(tiers[1].utilization, 34.0);
    }

    #[test]
    fn zhipu_invalid_percentage_falls_back_to_zero() {
        // percentage 为字符串或 null 时不应崩溃，按 0 处理（仍展示 tier，但用量为 0）。
        let data = json!({
            "limits": [
                { "type": "TOKENS_LIMIT", "percentage": "invalid", "nextResetTime": 1_000_000_000_000_i64 },
                { "type": "TOKENS_LIMIT", "percentage": null,      "nextResetTime": 2_000_000_000_000_i64 }
            ]
        });
        let tiers = parse_zhipu_token_tiers(&data);
        assert_eq!(tiers.len(), 2);
        assert_eq!(tiers[0].utilization, 0.0);
        assert_eq!(tiers[1].utilization, 0.0);
    }

    #[test]
    fn zhipu_extreme_percentage_values_pass_through() {
        // 负数 / 超 100 不做范围裁剪——下游渲染层负责显示策略，解析层只负责忠实搬运。
        let data = json!({
            "limits": [
                { "type": "TOKENS_LIMIT", "percentage": -5.0,  "nextResetTime": 1_000_000_000_000_i64 },
                { "type": "TOKENS_LIMIT", "percentage": 150.0, "nextResetTime": 2_000_000_000_000_i64 }
            ]
        });
        let tiers = parse_zhipu_token_tiers(&data);
        assert_eq!(tiers.len(), 2);
        assert_eq!(tiers[0].utilization, -5.0);
        assert_eq!(tiers[1].utilization, 150.0);
    }

    #[test]
    fn zhipu_more_than_two_token_limits_keeps_first_two() {
        // 防御性：智谱当前最多两条 TOKENS_LIMIT，若上游意外增加第三条应被丢弃，避免命名空缺。
        let data = json!({
            "limits": [
                { "type": "TOKENS_LIMIT", "percentage": 1.0, "nextResetTime": 1_000_000_000_000_i64 },
                { "type": "TOKENS_LIMIT", "percentage": 2.0, "nextResetTime": 2_000_000_000_000_i64 },
                { "type": "TOKENS_LIMIT", "percentage": 3.0, "nextResetTime": 3_000_000_000_000_i64 }
            ]
        });
        let tiers = parse_zhipu_token_tiers(&data);
        assert_eq!(tiers.len(), 2);
        assert_eq!(tiers[0].name, TIER_FIVE_HOUR);
        assert_eq!(tiers[1].name, TIER_WEEKLY_LIMIT);
    }
}
