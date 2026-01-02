// Admin route handlers for /ai/admin/*

use serde_json::json;
use worker::*;

use crate::ai::cloudflare::v1::rag;

// Handler for POST /ai/admin/rag-index
pub async fn handle_rag_index(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Only allow POST
    if req.method() != Method::Post {
        let headers = Headers::new();
        let _ = headers.set("Allow", "POST");
        return Response::error("Method not allowed", 405);
    }

    // Validate admin secret
    let admin_secret = req.headers().get("X-Admin-Secret")?;
    let rag_secret = ctx.env.var("RAG_SECRET")?.to_string();
    if admin_secret != Some(rag_secret.clone()) {
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
