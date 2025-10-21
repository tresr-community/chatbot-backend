use crate::console_trace;

use worker::*;

pub fn add_headers(response: &mut Response, origin: &str) -> Result<()> {
    // Add CORS headers to the response
    response
        .headers_mut()
        .set("X-XSS-Protection", "1; mode=block")?;

    response
        .headers_mut()
        .set("X-Content-Type-Options", "nosniff")?;

    response.headers_mut().set("X-Frame-Options", "DENY")?;

    response
        .headers_mut()
        .set("Referrer-Policy", "unsafe-url")?;

    response
        .headers_mut()
        .set("Permissions-Policy", "geolocation=(self), fullscreen=()")?;

    response
        .headers_mut()
        .set("Access-Control-Allow-Origin", origin)?;

    // Max-age is set to 1 day.
    response
        .headers_mut()
        .set("Cache-Control", "public, max-age=86400")?;

    // Add Vary header for proper caching.
    response.headers_mut().set("Vary", "Origin")?;

    response
        .headers_mut()
        .set("Content-Security-Policy", "frame-ancestors 'self'")?;

    Ok(())
}

pub async fn check_origin(req: &Request, ctx: &RouteContext<()>) -> Result<()> {
    // Extract the Origin header from the request.
    let binding = req.headers().get("Origin")?.unwrap_or_default();
    let origin: &str = binding.as_str();

    // If the Origin header is missing, return a 400 Bad Request response.
    if origin.is_empty() {
        return Err(Error::RustError(
            "Bad Request: 'Origin' header is missing".to_string(),
        ));
    }

    // Get the Allowed Origins from the Cloudflare Worker environment variables.
    let allowed_origins = ctx
        .var("ALLOWED_ORIGINS")
        .map_err(|_| Error::RustError("ERROR: ALLOWED_ORIGINS is not defined.".to_string()))?
        .to_string();

    if allowed_origins.contains(origin) {
        console_log!("INFO: Origin is allowed: {}", origin);
        Ok(())
    } else {
        console_error!("ERROR: Origin not allowed: {}", origin);
        Err(Error::RustError("Origin not allowed".to_string()))
    }
}

pub async fn preflight(req: &Request, ctx: &RouteContext<()>) -> Result<Response> {
    // Check the Origin is permitted before continuing.
    if check_origin(req, ctx).await.is_err() {
        console_error!("ERROR: Origin is forbidden");
        return Response::error("Forbidden", 403);
    }

    // Extract the Origin header from the request.
    let binding = req.headers().get("Origin")?.unwrap_or_default();
    let origin: &str = binding.as_str();

    // Add CORS headers to the response for OPTIONS requests.
    let headers = Headers::new();
    headers.set("Access-Control-Allow-Origin", origin)?;
    headers.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS")?;
    headers.set("Access-Control-Allow-Headers", "Content-Type")?;
    headers.set("Access-Control-Max-Age", "86400")?;
    headers.set("Vary", "Origin, Access-Control-Request-Headers")?;

    console_trace!("TRACE: Preflight response headers: {:?}", headers);

    Ok(Response::empty()?.with_headers(headers))
}
