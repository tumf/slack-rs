//! Debug logging helpers.
//!
//! This crate primarily prints user-facing progress messages to stdout.
//! Any verbose diagnostics should be gated behind an environment variable
//! and must never leak secrets (tokens, client secrets, etc.).

use serde_json::Value;

/// Debug level for output control
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DebugLevel {
    /// No debug output
    Off,
    /// Basic debug output (--debug)
    Debug,
    /// Verbose trace output (--trace)
    Trace,
}

/// Get the current debug level from environment variable or flags
///
/// Priority:
/// 1. --trace flag → Trace
/// 2. --debug flag → Debug
/// 3. SLACK_RS_DEBUG env → Debug
/// 4. Default → Off
pub fn get_debug_level(args: &[String]) -> DebugLevel {
    // Check for --trace flag
    if args.iter().any(|arg| arg == "--trace") {
        return DebugLevel::Trace;
    }

    // Check for --debug flag
    if args.iter().any(|arg| arg == "--debug") {
        return DebugLevel::Debug;
    }

    // Check environment variable
    if enabled() {
        return DebugLevel::Debug;
    }

    DebugLevel::Off
}

/// Returns true when debug logging is enabled.
///
/// Enable with `SLACK_RS_DEBUG=1` (also accepts: true/yes/on).
pub fn enabled() -> bool {
    match std::env::var("SLACK_RS_DEBUG") {
        Ok(v) => {
            let v = v.trim().to_ascii_lowercase();
            matches!(v.as_str(), "1" | "true" | "yes" | "on")
        }
        Err(_) => false,
    }
}

/// Print a debug line to stderr when enabled.
pub fn log(msg: impl AsRef<str>) {
    if enabled() {
        eprintln!("DEBUG: {}", msg.as_ref());
    }
}

/// Print debug information for API call context
///
/// Outputs to stderr when debug level is Debug or higher.
/// Never outputs secrets (tokens, client_secret).
pub fn log_api_context(
    level: DebugLevel,
    profile_name: Option<&str>,
    token_store_backend: &str,
    token_type: &str,
    method: &str,
    endpoint: &str,
) {
    if level >= DebugLevel::Debug {
        eprintln!("DEBUG: Profile: {}", profile_name.unwrap_or("<none>"));
        eprintln!("DEBUG: Token store: {}", token_store_backend);
        eprintln!("DEBUG: Token type: {}", token_type);
        eprintln!("DEBUG: API method: {}", method);
        eprintln!("DEBUG: Endpoint: {}", endpoint);
    }
}

/// Print trace-level debug information
///
/// Only outputs when debug level is Trace.
pub fn log_trace(level: DebugLevel, msg: impl AsRef<str>) {
    if level >= DebugLevel::Trace {
        eprintln!("TRACE: {}", msg.as_ref());
    }
}

/// Log Slack error code if present in response
///
/// Outputs to stderr when debug level is Debug or higher.
pub fn log_error_code(level: DebugLevel, response: &Value) {
    if level >= DebugLevel::Debug {
        if let Some(ok) = response.get("ok").and_then(|v| v.as_bool()) {
            if !ok {
                if let Some(error_code) = response.get("error").and_then(|v| v.as_str()) {
                    eprintln!("DEBUG: Slack error code: {}", error_code);
                }
            }
        }
    }
}

/// Returns a safe, non-reversible hint for a token.
///
/// Never returns any part of the token value.
pub fn token_hint(token: &str) -> String {
    let kind = if token.starts_with("xoxb-") {
        "xoxb"
    } else if token.starts_with("xoxp-") {
        "xoxp"
    } else if token.starts_with("xoxa-") {
        "xoxa"
    } else if token.starts_with("xoxr-") {
        "xoxr"
    } else if token.starts_with("xoxs-") {
        "xoxs"
    } else {
        "token"
    };

    format!("{} (len={})", kind, token.len())
}

/// Redact token-like values from a JSON string.
///
/// This is intentionally conservative: any string that looks like a Slack token
/// (starts with "xox") is replaced.
pub fn redact_json_secrets(json: &str) -> String {
    let Ok(mut v) = serde_json::from_str::<Value>(json) else {
        return "<non-json body>".to_string();
    };

    redact_value_in_place(&mut v);
    serde_json::to_string(&v).unwrap_or_else(|_| "<unserializable json>".to_string())
}

fn redact_value_in_place(v: &mut Value) {
    match v {
        Value::Object(map) => {
            for (_k, child) in map.iter_mut() {
                redact_value_in_place(child);
            }
        }
        Value::Array(items) => {
            for child in items.iter_mut() {
                redact_value_in_place(child);
            }
        }
        Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.starts_with("xox") {
                *s = "<redacted>".to_string();
            }
        }
        _ => {}
    }
}
