/*
///
/// # Overview
///
/// Handles the connection to the Cloudflare AI Gateway.
///
*/
pub mod google_ai_studio;
pub mod grok;
pub mod prompt;

use utils::console_trace;

use serde_json::json;
use worker::*;

// Refactored enum to use BoxFuture
enum AIResponse {
    GoogleAI(String),
    Grok(String),
    Invalid(Error),
}

impl AIResponse {
    // Helper method to get the response
    fn get_response(self) -> Result<String> {
        match self {
            AIResponse::GoogleAI(text) => Ok(text),
            AIResponse::Grok(text) => Ok(text),
            AIResponse::Invalid(error) => Err(error),
        }
    }
}

pub async fn handle_ai(
    message: &str,
    ai_type: &str,
    ctx: &RouteContext<()>,
) -> worker::Result<Response> {
    console_trace!(
        "TRACE: Handling request to AI backend '{}'",
        ai_type.to_string()
    );

    // Get the system prompt for the AI service.
    let prompt = prompt::get_system_prompt(ai_type)?;

    // Handle the response directly
    let ai_result: AIResponse = match ai_type {
        "google-ai-studio" => match google_ai_studio::call_ai(prompt, message, ctx).await {
            Ok(text) => AIResponse::GoogleAI(text),
            Err(e) => AIResponse::Invalid(e),
        },
        "grok" => match grok::call_ai(prompt, message, ctx).await {
            Ok(text) => AIResponse::Grok(text),
            Err(e) => AIResponse::Invalid(e),
        },
        _ => AIResponse::Invalid(Error::from(format!("Invalid AI type: {}", ai_type))),
    };

    // Get the response and construct the payload
    let payload = match ai_result.get_response() {
        Ok(text) => json!({
            "message": message,
            "reply": Some(text),
            "error": null,
        }),
        Err(e) => json!({
            "message": message,
            "reply": null,
            "error": e.to_string(),
        }),
    };

    let body = serde_json::to_string(&payload)?;
    let mut headers = Headers::new();
    headers.set("Content-Type", "application/json; charset=utf-8")?;

    Response::ok(body).map(|resp| resp.with_headers(headers))
}
