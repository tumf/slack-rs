//! Debug logging helpers.
//!
//! This crate primarily prints user-facing progress messages to stdout.
//! Any verbose diagnostics should be gated behind an environment variable
//! and must never leak secrets (tokens, client secrets, etc.).

use serde_json::Value;

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
