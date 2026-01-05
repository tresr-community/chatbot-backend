// Route handler for /health

use chatbot_utils::console_trace;

use worker::*;

/// Health check endpoint handler
/// Returns a simple "OK" response with appropriate headers
pub async fn handle_get(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    console_trace!("TRACE: Request received for route 'health'");

    // Set up response headers.
    let headers = Headers::new();
    headers.set("Content-Type", "text/plain; charset=utf-8")?;
    headers.set("Cache-Control", "no-store, no-cache, must-revalidate")?;
    headers.set("Pragma", "no-cache")?;
    headers.set("X-Content-Type-Options", "nosniff")?;
    headers.set("X-Frame-Options", "DENY")?;
    headers.set("X-XSS-Protection", "1; mode=block")?;

    // Return a simple OK response
    Ok(Response::ok("OK")?.with_headers(headers))
}
