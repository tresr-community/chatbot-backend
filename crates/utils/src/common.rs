use crate::console_trace;

use serde_json::from_str;
use worker::*;

/// Validates an environment variable.
/// Ensures it is defined, and not empty.
pub async fn validate_env_var(
    ctx: &RouteContext<()>,
    variable: &str,
    variable_type: &str,
) -> Result<String> {
    // The variable type must be either 'json' or 'string'.
    // If it's not provided, default to 'string'.
    let variable_type = variable_type.to_lowercase();
    let variable_type = if variable_type == "json" {
        "json"
    } else {
        "string"
    };

    // Check 1. Is the environment variable defined?
    let env_var = ctx
        .var(variable)
        .map_err(|_| Error::RustError(format!("{} is not defined.", variable)))?;

    // Check 2. Is the environment variable empty?
    if env_var.to_string().trim().is_empty() {
        return Err(Error::RustError(format!("{} is empty.", variable)));
    }

    // If the variable type is 'json', validate it as JSON.
    if variable_type == "json" {
        // Check 3. Is the environment variable valid JSON?
        from_str::<serde_json::Value>(&env_var.to_string())
            .map_err(|_| Error::RustError(format!("Failed to parse {} as JSON.", variable)))?;
    }

    // If all checks pass, return a success message.
    Ok(format!("{} validated successfully.", variable))
}

/// Parses the path to extract the AI version and backend.
/// The path should be in the format: /api/{version}/{backend}
pub fn parse_path(path: &str) -> Result<(String, String)> {
    // Split the path into parts.
    let parts: Vec<&str> = path.split('/').collect();

    // Check that the path has at least 3 parts.
    if parts.len() < 3 {
        return Err(Error::RustError("Invalid path.".to_string()));
    }

    // Extract the version and backend from the path.
    let version = parts[2].to_string();
    let backend = parts[3].to_string();

    console_trace!("Split path: {:?}", parts);
    console_trace!("Version: {}", version);
    console_trace!("Backend: {}", backend);

    Ok((version, backend))
}
