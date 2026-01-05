// Route handler for /admin/*

use serde_json::json;
use worker::*;

use std::collections::HashMap;

use chatbot_utils::console_trace;

use crate::ai::cloudflare::v1::rag;

use chatbot_utils::common::validate_env_var;

use chatbot_utils::cors::{check_origin, preflight};

// Handler for POST /admin/rag-index
pub async fn handle_rag_index(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    console_trace!("TRACE: Request received for route 'admin/rag-index'");

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

    // Define the required environment variables.
    let mut required_variables = HashMap::new();
    required_variables.insert("RAG_SECRET", "string"); // Cloudflare Vectorize secret

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

    // TODO: clean this up to be consistent.
    // Validate admin secret
    let admin_secret = req.headers().get("X-Admin-Secret")?;
    let rag_secret = ctx.env.var("RAG_SECRET")?.to_string();
    if admin_secret != Some(rag_secret.clone()) {
        console_error!("ERROR: Unauthorized access attempt to admin route");
        return Response::error("Unauthorized", 401);
    }

    // Call the indexing function
    match rag::index_docs(&ctx.env).await {
        Ok(_) => {
            let body = json!({
                "success": true,
                "message": "RAG indexing completed successfully"
            })
            .to_string();
            let headers = Headers::new();
            let _ = headers.set("Content-Type", "application/json");
            Response::ok(body).map(|resp| resp.with_headers(headers))
        }
        Err(e) => {
            console_error!("RAG index error: {}", e);
            Response::from_json(&json!({
                "success": false,
                "message": "Indexing failed"
            }))
        }
    }
}
