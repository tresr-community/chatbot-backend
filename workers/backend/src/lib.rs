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

use utils::console_trace;
use utils::handler::handle_404;

use routes::*;
use worker::*;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_trace!("TRACE: Request received for worker 'api'");

    let path = req.path();

    if path.starts_with("/api/") {
        return Router::new()
            .post_async(path.as_str(), api::handle_all)
            .run(req, env)
            .await;
    }

    Router::new()
        .get_async("/health", health::handle_get)
        .get_async("/quotes", quotes::handle_get)
        .or_else_any_method_async("/", handle_404)
        .run(req, env)
        .await
}
