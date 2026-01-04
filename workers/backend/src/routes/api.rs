// Route handlers for /*

use crate::ai;

use chatbot_utils::common::{parse_path, validate_env_var};
use chatbot_utils::console_trace;
use chatbot_utils::cors::{add_headers, check_origin, preflight};

use serde::Deserialize;
use std::collections::HashMap;
use worker::*;

#[derive(Deserialize)]
struct JsonPayload {
    message: String,
}

// handle_all serves all requests to the /* route.
// The function expects the path to be in the following format.
// /{api_version}/{ai_backend}
// The function will parse the path to extract the version and AI backend service.
// For any non-existent or invalid path, the function will return a 404 Not Found response.
pub async fn handle_all(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    console_trace!("TRACE: Request received for route 'api'");

    let path = req.path();

    // CORS pre-flight check.
    if req.method() == Method::Options {
        console_log!("INFO: Running CORS pre-flight check.");
        return preflight(&req, &ctx).await;
    }

    // Only POST method are allowed for AI services at this time.
    // If the method is not POST, return a 405 Method Not Allowed response.
    if req.method() != Method::Post {
        console_error!("ERROR: Method {:?} not allowed", req.method());
        return Response::error("Method Not Allowed", 405);
    }

    // Extract the Origin header from the request.
    let binding = req.headers().get("Origin")?.unwrap_or_default();
    let origin: &str = binding.as_str();

    // If the Origin header is missing, return a 400 Bad Request response.
    if origin.is_empty() {
        console_error!("ERROR: Origin is missing from headers");
        return Response::error("Bad Request: 'Origin' header is missing", 400);
    }

    // Ensure the origin is on the allowed list before proceeding.
    if check_origin(&req, &ctx).await.is_err() {
        console_error!("ERROR: Origin is forbidden {}", origin);
        return Response::error("Forbidden", 403);
    }

    // Parse the ai version and backend from the path.
    let (ai_version, ai_backend) = match parse_path(path.as_str()) {
        Ok((version, backend)) => (version, backend),
        Err(e) => {
            console_error!("ERROR: {}", e);
            return Response::error("API version and AI backend not found", 404);
        }
    };
    console_trace!("TRACE: AI Version: {}", ai_version.as_str());
    console_trace!("TRACE: AI Backend: {}", ai_backend.as_str());

    // Ensure that the AI backend and version are in the allowed whitelist.
    if !ai::is_allowed_ai_backend(&ctx, &ai_backend, &ai_version) {
        console_error!(
            "ERROR: AI backend and version combination is not allowed {} {}",
            ai_backend,
            ai_version
        );
        return Response::error("Forbidden", 403);
    } else {
        console_log!(
            "INFO: AI backend and version combination is allowed {} {}",
            ai_backend,
            ai_version
        );
    }

    // Define the required environment variables.
    let mut required_variables = HashMap::new();
    required_variables.insert("AI_SECRET", "string"); // Shared secret with frontend/backend

    // Validate required environment variables.
    for (name, type_str) in required_variables.iter() {
        // Validate the environment variable.
        let result = validate_env_var(&ctx, name, type_str).await;

        // If the validation fails, return an error response.
        if let Err(err) = &result {
            console_error!("ERROR: {}", err);
            return Response::error(
                format!(
                    "Internal Server Error: Failed to validate required variable {}. {}",
                    name, err
                ),
                500,
            );
        } else {
            console_log!("INFO: {}", result.unwrap());
        }
    }

    // Deserialize JSON body.
    let body = req
        .clone_mut()
        .unwrap()
        .json::<JsonPayload>()
        .await
        .map_err(|e| {
            console_error!("ERROR: Failed to deserialize JSON request body {}", e);
            Response::error(
                format!("Failed to deserialize JSON request body {}", e),
                400,
            )
        });

    // Get the message from the JSON body.
    let message = body.unwrap().message;

    // If the message is empty, return a 400 Bad Request response.
    if message.is_empty() {
        console_error!("ERROR: Message is missing from body");
        return Response::error("Bad Request: 'message' data is missing", 400);
    }

    // Extract Authorization header.
    let auth_opt = req.headers().get("Authorization");
    let auth_header = match auth_opt {
        Ok(Some(header)) => header,
        _ => String::new(), // None/missing/err → empty.
    };

    let ai_secret = auth_header
        .strip_prefix("Bearer ")
        .map(|stripped| stripped.to_owned())
        .unwrap_or(auth_header);

    if ai_secret.is_empty() {
        console_error!("ERROR: Unauthorized - missing/invalid Authorization");
        return Response::error("Unauthorized", 401);
    }
    console_trace!("TRACE: AI_SECRET validated (length {})", ai_secret.len());

    // Make the AI service call to the selected backend and get the response.
    let mut ai_response =
        ai::handle_ai_service(&message, &ai_secret, &ai_backend, &ai_version, ctx).await?;

    // Add CORS headers to the response and return.
    add_headers(&mut ai_response, origin)?;
    Ok(ai_response)
}
