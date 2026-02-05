//! Local callback server for OAuth flow
//!
//! Runs a temporary HTTP server on localhost to receive the OAuth callback

use super::types::OAuthError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::time::{timeout, Duration};

#[derive(Debug, Clone)]
pub struct CallbackResult {
    pub code: String,
    #[allow(dead_code)]
    pub state: String,
}

/// Run a local HTTP server to receive OAuth callback
///
/// Returns the authorization code and state received from the callback
///
/// # Arguments
/// * `port` - Port to listen on (typically 3000)
/// * `expected_state` - Expected state value for CSRF verification
/// * `timeout_secs` - Timeout in seconds (default 300)
pub async fn run_callback_server(
    port: u16,
    expected_state: String,
    timeout_secs: u64,
) -> Result<CallbackResult, OAuthError> {
    let bind_addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&bind_addr)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Failed to bind to port {}: {}", port, e)))?;

    let actual_port = listener.local_addr().map(|a| a.port()).unwrap_or(port);
    println!(
        "Listening for OAuth callback on http://127.0.0.1:{}",
        actual_port
    );

    let result: Arc<Mutex<Option<Result<CallbackResult, OAuthError>>>> = Arc::new(Mutex::new(None));

    let server_result = result.clone();
    let server_task = async move {
        loop {
            let (mut socket, _) = match listener.accept().await {
                Ok(conn) => conn,
                Err(e) => {
                    let mut res = server_result.lock().unwrap();
                    *res = Some(Err(OAuthError::ServerError(format!(
                        "Failed to accept connection: {}",
                        e
                    ))));
                    break;
                }
            };

            let mut buffer = vec![0; 4096];
            let n = match socket.read(&mut buffer).await {
                Ok(n) if n > 0 => n,
                _ => continue,
            };

            let request = String::from_utf8_lossy(&buffer[..n]);

            // Parse the request line
            if let Some(first_line) = request.lines().next() {
                if let Some(path_part) = first_line.split_whitespace().nth(1) {
                    if let Some(query_start) = path_part.find('?') {
                        let query = &path_part[query_start + 1..];
                        let params = parse_query_string(query);

                        let response = if let (Some(code), Some(state)) =
                            (params.get("code"), params.get("state"))
                        {
                            // Verify state
                            if state != &expected_state {
                                let mut res = server_result.lock().unwrap();
                                *res = Some(Err(OAuthError::StateMismatch {
                                    expected: expected_state.clone(),
                                    actual: state.clone(),
                                }));
                                create_error_response("State mismatch - possible CSRF attack")
                            } else {
                                let mut res = server_result.lock().unwrap();
                                *res = Some(Ok(CallbackResult {
                                    code: code.clone(),
                                    state: state.clone(),
                                }));
                                create_success_response()
                            }
                        } else if let Some(error) = params.get("error") {
                            let mut res = server_result.lock().unwrap();
                            *res = Some(Err(OAuthError::SlackError(error.clone())));
                            create_error_response(&format!("OAuth error: {}", error))
                        } else {
                            create_error_response("Missing required parameters")
                        };

                        let _ = socket.write_all(response.as_bytes()).await;
                        let _ = socket.flush().await;
                        break;
                    }
                }
            }
        }
    };

    // Run with timeout
    match timeout(Duration::from_secs(timeout_secs), server_task).await {
        Ok(_) => {
            let res = result.lock().unwrap();
            match res.as_ref() {
                Some(Ok(callback_result)) => Ok(callback_result.clone()),
                Some(Err(e)) => Err(format_oauth_error(e)),
                None => Err(OAuthError::ServerError("No result received".to_string())),
            }
        }
        Err(_) => Err(OAuthError::ServerError(format!(
            "Timeout after {} seconds waiting for callback",
            timeout_secs
        ))),
    }
}

/// Helper function to format OAuthError for re-creation
fn format_oauth_error(err: &OAuthError) -> OAuthError {
    match err {
        OAuthError::ConfigError(msg) => OAuthError::ConfigError(msg.clone()),
        OAuthError::NetworkError(msg) => OAuthError::NetworkError(msg.clone()),
        OAuthError::HttpError(code, msg) => OAuthError::HttpError(*code, msg.clone()),
        OAuthError::ParseError(msg) => OAuthError::ParseError(msg.clone()),
        OAuthError::SlackError(msg) => OAuthError::SlackError(msg.clone()),
        OAuthError::StateMismatch { expected, actual } => OAuthError::StateMismatch {
            expected: expected.clone(),
            actual: actual.clone(),
        },
        OAuthError::ServerError(msg) => OAuthError::ServerError(msg.clone()),
        OAuthError::BrowserError(msg) => OAuthError::BrowserError(msg.clone()),
    }
}

/// Parse URL query string into a HashMap
fn parse_query_string(query: &str) -> HashMap<String, String> {
    query
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.split('=');
            match (parts.next(), parts.next()) {
                (Some(key), Some(value)) => Some((
                    key.to_string(),
                    urlencoding::decode(value).ok()?.to_string(),
                )),
                _ => None,
            }
        })
        .collect()
}

fn create_success_response() -> String {
    "HTTP/1.1 200 OK\r\n\
     Content-Type: text/html; charset=utf-8\r\n\
     Connection: close\r\n\
     \r\n\
     <html>\
     <head><title>Authentication Successful</title></head>\
     <body>\
     <h1>✓ Authentication Successful</h1>\
     <p>You can close this window and return to the CLI.</p>\
     </body>\
     </html>"
        .to_string()
}

fn create_error_response(message: &str) -> String {
    format!(
        "HTTP/1.1 400 Bad Request\r\n\
         Content-Type: text/html; charset=utf-8\r\n\
         Connection: close\r\n\
         \r\n\
         <html>\
         <head><title>Authentication Failed</title></head>\
         <body>\
         <h1>✗ Authentication Failed</h1>\
         <p>{}</p>\
         </body>\
         </html>",
        message
    )
}

// Note: urlencoding is used for URL decoding
// We need to add this dependency
mod urlencoding {
    pub fn decode(s: &str) -> Result<String, ()> {
        // Simple URL decode implementation
        let mut result = String::new();
        let mut chars = s.chars();
        while let Some(c) = chars.next() {
            match c {
                '%' => {
                    let hex: String = chars.by_ref().take(2).collect();
                    if hex.len() == 2 {
                        if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                            result.push(byte as char);
                        } else {
                            return Err(());
                        }
                    } else {
                        return Err(());
                    }
                }
                '+' => result.push(' '),
                c => result.push(c),
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_string() {
        let query = "code=test_code&state=test_state&foo=bar";
        let params = parse_query_string(query);

        assert_eq!(params.get("code"), Some(&"test_code".to_string()));
        assert_eq!(params.get("state"), Some(&"test_state".to_string()));
        assert_eq!(params.get("foo"), Some(&"bar".to_string()));
    }

    #[test]
    fn test_parse_query_string_with_encoding() {
        let query = "message=hello+world&name=test%20user";
        let params = parse_query_string(query);

        assert_eq!(params.get("message"), Some(&"hello world".to_string()));
        assert_eq!(params.get("name"), Some(&"test user".to_string()));
    }

    #[tokio::test]
    async fn test_callback_server_timeout() {
        // Test that the server times out appropriately
        let state = "test_state".to_string();
        // Use an ephemeral port to avoid test flakiness from port conflicts.
        let result = run_callback_server(0, state, 1).await;

        assert!(result.is_err());
        match result {
            Err(OAuthError::ServerError(msg)) => {
                assert!(msg.contains("Timeout"));
            }
            _ => panic!("Expected ServerError with timeout"),
        }
    }
}
