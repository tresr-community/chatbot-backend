use crate::console_trace;

use worker::*;

// A simple route that returns a 404 status code.
pub async fn handle_404(_req: Request, _ctx: RouteContext<()>) -> worker::Result<Response> {
    Response::error("Not Found", 404)
}

// Extracts the given query parameter from the URL.
// If the parameter is not found, the default value is returned instead.
pub async fn extract_query_param(url: &Url, param: &str, default: &str) -> String {
    // Extract the given query parameter or handle if empty.
    let query_params = url.query_pairs();

    if query_params.count() == 0 {
        return default.to_string();
    }

    let param_value = query_params
        .into_iter()
        .find(|(key, _value)| key == param)
        .map_or_else(|| default.to_string(), |(_key, value)| value.to_string());

    console_trace!(
        "TRACE: Extracted query parameter: {}={}",
        param,
        param_value
    );

    param_value
}
