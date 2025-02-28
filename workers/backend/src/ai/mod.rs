///
/// mod.rs
///
/// # Overview
///
/// Serves the different AI types for the Chatbot.
///
pub mod chatbot;
pub mod cloudflare;

use chatbot_utils::console_trace;

use serde_json::Value;
use worker::*;

pub fn is_allowed_ai_backend(ctx: &RouteContext<()>, ai_backend: &str, ai_version: &str) -> bool {
    // Get the enabled backends from environment
    let enabled_backends = match ctx.var("ENABLED_AI_BACKENDS") {
        Ok(var) => var.to_string(),
        Err(_) => return false,
    };

    // Parse the JSON string
    let backends: Value = match serde_json::from_str(&enabled_backends) {
        Ok(json) => json,
        Err(_) => return false,
    };

    // Check if the backend exists and contains the version provided
    if let Some(backend_map) = backends["backends"].as_object() {
        if let Some(versions) = backend_map.get(ai_backend) {
            if let Some(version_array) = versions.as_array() {
                return version_array
                    .iter()
                    .any(|v| v.as_str().map(|s| s == ai_version).unwrap_or(false));
            }
        }
    }

    false
}

pub async fn handle_ai_service(
    message: &str,
    ai_secret: &str,
    ai_backend: &str,
    ai_version: &str,
    ctx: RouteContext<()>,
) -> worker::Result<Response> {
    // Get the AI Secret from the Cloudflare Worker environment variables.
    let ai_secret_env = ctx
        .var("AI_SECRET")
        .map_err(|_| Error::RustError("ERROR: AI_SECRET is not defined.".to_string()))?
        .to_string();

    // Confirm that the provided AI Secret matches the environment variable.
    if ai_secret != ai_secret_env {
        console_error!("ERROR: Unauthorized");
        return Response::error("Unauthorized", 401);
    }

    // Convert the AI backend to lowercase for comparison.
    let ai_backend = ai_backend.to_lowercase();

    // Convert the AI version to lowercase for comparison.
    let ai_version = ai_version.to_lowercase();

    console_trace!("TRACE: AI Backend: {}", ai_backend);
    console_trace!("TRACE: AI Version: {}", ai_version);

    // Build the request in the correct format for the selected backend.
    // Cloudflare AI Gateway is used for all AI services except Chatbot.
    let result = match (ai_version.as_str(), ai_backend.as_str()) {
        // Quotes chatbot
        ("v1", "chatbot") => chatbot::v1::handle_ai(message).await,

        // Google AI Studio
        ("v1", "google-ai-studio") => {
            cloudflare::v1::handle_ai(message, "google-ai-studio", &ctx).await
        }

        // XAI Grok
        ("v1", "grok") => cloudflare::v1::handle_ai(message, "grok", &ctx).await,

        _ => Response::error(
            format!("AI backend {} has no version {}", ai_backend, ai_version),
            404,
        ),
    };

    // Return the result from the AI backend.
    result
}
