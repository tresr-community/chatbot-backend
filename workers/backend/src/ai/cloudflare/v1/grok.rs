/*
///
/// X Grok
///
/// Example:
///
/// curl -X POST https://gateway.ai.cloudflare.com/v1/6d6a8fb1f9f3f38b7374d4974c0743cf/tresr-community/grok/v1/chat/completions \
/// --header 'content-type: application/json' \
/// --header 'cf-aig-authorization: Bearer CF_AI_GATEWAY_TOKEN' \
/// --header 'Authorization: Bearer GROK_TOKEN' \
/// --data '{
///    "model": "grok-2-1212",
///    "messages": [
///        {
///            "role": "user",
///            "content": "What is Cloudflare?"
///        }
///    ]
///}'
///
*/
use crate::wasm_bindgen::JsValue;
use serde::Deserialize;
use serde_json::json;
use worker::*;

use chatbot_utils::console_trace;

const AI_ENDPOINT: &str =
    "https://gateway.ai.cloudflare.com/v1/6d6a8fb1f9f3f38b7374d4974c0743cf/tresr-community/grok";
const AI_MODEL: &str = "grok-2-1212";

// Response structures matching Grok's format
#[derive(Deserialize)]
struct GrokResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
    //finish_reason: String,
}

#[derive(Deserialize)]
struct Message {
    //role: String,
    content: String,
}

pub async fn call_ai(
    prompt: &str,
    message: &str,
    ctx: &RouteContext<()>,
) -> worker::Result<String> {
    console_trace!("TRACE: Calling Grok AI");

    // Get the Cloudflare AI Gateway token
    console_trace!("TRACE: Retrieving Cloudflare AI Gateway token");
    let ai_gateway_token = ctx
        .var("CF_AI_GATEWAY_TOKEN")
        .map_err(|e| {
            console_error!("ERROR: Failed to get CF_AI_GATEWAY_TOKEN: {}", e);
            Error::from(e.to_string())
        })?
        .to_string();

    // Get the Grok API key
    console_trace!("TRACE: Retrieving Grok API token");
    let api_key = ctx
        .var("GROK_TOKEN")
        .map_err(|e| {
            console_error!("ERROR: Failed to get GROK_TOKEN: {}", e);
            Error::from(e.to_string())
        })?
        .to_string();

    // Log payload construction
    console_trace!("TRACE: Constructing request payload");
    let payload = json!({
        "model": AI_MODEL,
        "messages": [
            {
                "role": "system",
                "content": prompt
            },
            {
                "role": "user",
                "content": message
            }
        ]
    });
    console_trace!("TRACE: Payload constructed: {}", payload.to_string());

    // Log headers setup
    console_trace!("TRACE: Setting up request headers");
    let mut headers = Headers::new();
    headers.set("Content-Type", "application/json")?;
    headers.set(
        "cf-aig-authorization",
        &format!("Bearer {}", ai_gateway_token),
    )?;
    headers.set("Authorization", &format!("Bearer {}", api_key))?;

    let payload_str = payload.to_string();

    // Log request preparation
    let url = format!("{}/v1/chat/completions", AI_ENDPOINT);
    console_trace!("TRACE: Preparing request to URL: {}", url);

    // Create the request init with headers
    let mut init = RequestInit::new();
    init.with_method(Method::Post);
    init.with_headers(headers);
    init.with_body(Some(JsValue::from_str(&payload_str)));

    // Log request attempt
    console_trace!("TRACE: Sending request to Grok");
    let mut response = Fetch::Request(Request::new_with_init(&url, &init)?)
        .send()
        .await
        .map_err(|e| {
            console_error!("ERROR: Failed to send request: {}", e);
            Error::from(e.to_string())
        })?;

    // Log response status
    console_trace!(
        "TRACE: Received response with status: {}",
        response.status_code()
    );

    if !(200..=299).contains(&response.status_code()) {
        let error_text = response.text().await.unwrap_or_default();
        console_error!(
            "ERROR: API request failed with status {}: {}",
            response.status_code(),
            error_text
        );
        return Err(Error::from(format!(
            "API request failed: {} - {}",
            response.status_code(),
            error_text
        )));
    }

    // Log response parsing
    console_trace!("TRACE: Parsing response");
    let response_text = response.text().await.map_err(|e| {
        console_error!("ERROR: Failed to get response text: {}", e);
        Error::from(e.to_string())
    })?;

    console_trace!("TRACE: Raw response: {}", response_text);

    let grok_response: GrokResponse = serde_json::from_str(&response_text).map_err(|e| {
        console_error!("ERROR: Failed to parse response JSON: {}", e);
        Error::from(format!("Failed to parse Grok response: {}", e))
    })?;

    // Get the first choice's message content
    let response_content = grok_response
        .choices
        .first()
        .map(|choice| choice.message.content.clone())
        .ok_or_else(|| Error::from("No response content from Grok"))?;

    console_trace!("TRACE: Successfully processed response");
    Ok(response_content)
}
