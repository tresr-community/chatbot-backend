/*
///
/// Google AI Studio
///
/// Example:
///
/// curl --request POST \
///     --url https://gateway.ai.cloudflare.com/v1/6d6a8fb1f9f3f38b7374d4974c0743cf/nfttreasure-community/google-ai-studio/v1/models/gemini-1.0-pro:generateContent \
///     --header 'content-type: application/json' \
///     --header 'cf-aig-authorization: Bearer CF_AI_GATEWAY_TOKEN' \
///     --header 'x-goog-api-key: GOOGLE_AI_STUDIO_TOKEN' \
///     --data '{"contents":[{"role":"user","parts":[{"text":"What is Cloudflare?"}]}]}
///
*/
use crate::wasm_bindgen::JsValue;
use serde::Deserialize;
use serde_json::json;
use worker::*;

use chatbot_utils::console_trace;

const AI_ENDPOINT: &str = "https://gateway.ai.cloudflare.com/v1/6d6a8fb1f9f3f38b7374d4974c0743cf/tresr-community/google-ai-studio";
const AI_MODEL: &str = "gemini-1.5-pro";

// Response structures matching Google AI's format
#[derive(Deserialize)]
struct GoogleAIResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Content,
    //finish_reason: String,
}

#[derive(Deserialize)]
struct Content {
    parts: Vec<Part>,
    //role: String,
}

#[derive(Deserialize)]
struct Part {
    text: String,
}

pub async fn call_ai(
    prompt: &str,
    message: &str,
    ctx: &RouteContext<()>,
) -> worker::Result<String> {
    console_trace!("TRACE: Calling Google AI Studio");

    // Get the Cloudflare AI Gateway token
    console_trace!("TRACE: Retrieving Cloudflare AI Gateway token");
    let ai_gateway_token = ctx
        .var("CF_AI_GATEWAY_TOKEN")
        .map_err(|e| {
            console_error!("ERROR: Failed to get CF_AI_GATEWAY_TOKEN: {}", e);
            Error::from(e.to_string())
        })?
        .to_string();

    // Get the Google AI Studio API key
    console_trace!("TRACE: Retrieving Google AI Studio API key");
    let api_key = ctx
        .var("GOOGLE_AI_STUDIO_TOKEN")
        .map_err(|e| {
            console_error!("ERROR: Failed to get GOOGLE_AI_STUDIO_TOKEN: {}", e);
            Error::from(e.to_string())
        })?
        .to_string();

    // Log payload construction
    console_trace!("TRACE: Constructing request payload");
    let payload = json!({
        "contents": [{
            "role": "system",
            "parts": [{
                "text": prompt
            }],
            "role": "user",
            "parts": [{
                "text": message
            }]
        }]
    });
    console_trace!("TRACE: Payload constructed: {}", payload.to_string());

    // Log headers setup
    console_trace!("TRACE: Setting up request headers");
    let headers = Headers::new();
    headers.set("Content-Type", "application/json")?;
    headers.set(
        "cf-aig-authorization",
        &format!("Bearer {}", ai_gateway_token),
    )?;
    headers.set("x-goog-api-key", &api_key)?;

    let payload_str = payload.to_string();

    // Log request preparation
    let url = format!("{}/v1/models/{}:generateContent", AI_ENDPOINT, AI_MODEL);
    console_trace!("TRACE: Preparing request to URL: {}", url);

    // Create the request init with headers
    let mut init = RequestInit::new();
    init.with_method(Method::Post);
    init.with_headers(headers); // Use the original headers
    init.with_body(Some(JsValue::from_str(&payload_str)));

    // Log request attempt
    console_trace!("TRACE: Sending request to Google AI Studio");
    let mut response = Fetch::Request(Request::new_with_init(&url, &init)?) // Pass a reference to init
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

    let ai_response: GoogleAIResponse = serde_json::from_str(&response_text).map_err(|e| {
        console_error!("ERROR: Failed to parse response JSON: {}", e);
        Error::from(format!("Failed to parse AI response: {}", e))
    })?;

    // Combine and return response
    let combined_response: String = ai_response
        .candidates
        .iter()
        .flat_map(|candidate| candidate.content.parts.iter().map(|part| part.text.clone()))
        .collect::<Vec<String>>()
        .join("\n");

    console_trace!("TRACE: Successfully processed response");
    Ok(combined_response)
}
