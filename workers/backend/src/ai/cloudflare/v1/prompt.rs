///
/// mod.rs
///
/// # Overview
///
/// A mapping of the user friendly name to the system prompt in use.
///
use chatbot_utils::console_trace;

use worker::*;

// TODO: Improve this static prompt.
static PROMPT: &str = "
    You are a helpful assistant named TestGPT.

    When asked your name, you should respond with 'TestGPT'.
    ";

pub fn get_system_prompt(ai_type: &str) -> worker::Result<&str> {
    console_trace!(
        "TRACE: Getting system prompt for AI type '{}'",
        ai_type.to_string()
    );

    // TODO: Implement a mapping of AI types to system prompts.
    // For now, we'll just return a single prompt for all AI types.
    Ok(PROMPT)
}
