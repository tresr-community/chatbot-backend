///
/// # API Worker
///
/// lib.rs
///
/// ## Overview
///
/// Entry point for the Cloudflare Worker that handles API requests.
///
mod ai;
mod routes;

use chatbot_utils::console_trace;
use chatbot_utils::handler::handle_404;

use routes::*;
use worker::*;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_trace!("TRACE: Request received for worker 'backend'");

    let path = req.path();

    // Index the Vector Database
    if path.starts_with("/ai/admin/rag-index") {
        return Router::new()
            .post_async(path.as_str(), admin::handle_rag_index)
            .run(req, env)
            .await;
    }

    // Handle API requests
    if path.starts_with("/ai/") {
        return Router::new()
            .post_async(path.as_str(), api::handle_all)
            .run(req, env)
            .await;
    }

    // Handle health checks
    Router::new()
        .get_async("/health", health::handle_get)
        .or_else_any_method_async("/", handle_404)
        .run(req, env)
        .await
}
