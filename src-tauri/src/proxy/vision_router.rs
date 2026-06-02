use serde_json::Value;

/// CC-Gateway-Pro: Vision Model Auto-Routing
///
/// 从 handler_context.rs 中抽取，避免上游同步覆盖。
/// 检测请求中的图片内容，自动切换到 provider 配置的 vision model。
///
/// 用法:
///   let effective_model = vision_router::route(request_model, body, provider, tag);
///   if effective_model != request_model {
///       body["model"] = json!(effective_model);
///   }
/// 检查请求体是否包含图片内容
pub fn has_image_content(body: &Value) -> bool {
    crate::proxy::model_mapper::ModelMapping::has_image_content(body)
}

/// 执行 vision routing：
/// 1. 检查 body 是否包含图片
/// 2. 如果有图片且 provider 配置了 vision_model，切换到 vision_model
/// 3. 返回最终的模型名（如果切换了则同步更新 body["model"]）
///
/// 返回: (effective_model, did_switch)
pub fn route(
    request_model: &str,
    body: &mut Value,
    provider: &crate::provider::Provider,
    tag: &str,
) -> String {
    let has_images = has_image_content(body);
    let vision_model = provider
        .meta
        .as_ref()
        .and_then(|m| m.vision_model.as_deref());

    log::info!(
        "[{}] Vision check: has_images={}, provider={}, vision_model={:?}",
        tag,
        has_images,
        provider.name,
        vision_model
    );

    if !has_images {
        return request_model.to_string();
    }

    match vision_model {
        Some(vision) if !vision.is_empty() => {
            log::info!(
                "[{}] Vision routing: detected image content, switching model {} -> {} (provider: {})",
                tag,
                request_model,
                vision,
                provider.name
            );
            // 同步到 body，让 forwarder 的 model mapping 基于 vision-routed 模型做映射
            body["model"] = serde_json::json!(vision);
            vision.to_string()
        }
        _ => {
            log::warn!(
                "[{}] Vision routing: image detected but provider {} has no vision_model configured, using default model",
                tag,
                provider.name
            );
            request_model.to_string()
        }
    }
}
