//! Ngrok tunnel integration for OAuth redirect URI
//!
//! Provides functionality to start ngrok tunnel, extract public URL,
//! and stop the tunnel after OAuth flow completion.

use regex::Regex;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::Duration;

/// Error type for ngrok operations
#[derive(Debug)]
#[allow(dead_code)]
pub enum NgrokError {
    /// Failed to start ngrok process
    StartError(String),
    /// Failed to extract public URL from ngrok output
    UrlExtractionError(String),
    /// Process terminated unexpectedly
    ProcessTerminated(String),
}

impl std::fmt::Display for NgrokError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NgrokError::StartError(msg) => write!(f, "Failed to start ngrok: {}", msg),
            NgrokError::UrlExtractionError(msg) => {
                write!(f, "Failed to extract URL: {}", msg)
            }
            NgrokError::ProcessTerminated(msg) => write!(f, "Process terminated: {}", msg),
        }
    }
}

impl std::error::Error for NgrokError {}

/// Ngrok tunnel manager
#[allow(dead_code)]
pub struct NgrokTunnel {
    process: Child,
    public_url: String,
}

impl NgrokTunnel {
    /// Start ngrok tunnel and extract public URL
    ///
    /// # Arguments
    /// * `ngrok_path` - Path to ngrok executable (or "ngrok" to use PATH)
    /// * `port` - Local port to tunnel (e.g., 8765)
    /// * `timeout_secs` - Timeout in seconds to wait for URL extraction
    ///
    /// # Returns
    /// NgrokTunnel instance with running process and extracted public URL
    #[allow(dead_code)]
    pub fn start(ngrok_path: &str, port: u16, timeout_secs: u64) -> Result<Self, NgrokError> {
        // Start ngrok process
        let mut process = Command::new(ngrok_path)
            .args(["http", &port.to_string()])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                NgrokError::StartError(format!(
                    "Failed to execute '{}': {}. Make sure ngrok is installed and accessible.",
                    ngrok_path, e
                ))
            })?;

        // Extract stdout and stderr
        let stdout = process
            .stdout
            .take()
            .ok_or_else(|| NgrokError::StartError("Failed to capture stdout".to_string()))?;
        let stderr = process
            .stderr
            .take()
            .ok_or_else(|| NgrokError::StartError("Failed to capture stderr".to_string()))?;

        // Create channel for URL extraction
        let (tx, rx): (std::sync::mpsc::Sender<String>, Receiver<String>) = channel();

        // Spawn thread to read stdout and stderr
        let tx_clone = tx.clone();
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                if let Some(url) = extract_public_url(&line) {
                    let _ = tx_clone.send(url);
                    break;
                }
            }
        });

        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                if let Some(url) = extract_public_url(&line) {
                    let _ = tx.send(url);
                    break;
                }
            }
        });

        // Wait for URL with timeout
        let public_url = rx
            .recv_timeout(Duration::from_secs(timeout_secs))
            .map_err(|_| {
                NgrokError::UrlExtractionError(format!(
                    "Timeout waiting for ngrok URL (waited {} seconds). \
                     Make sure ngrok is working correctly.",
                    timeout_secs
                ))
            })?;

        Ok(Self {
            process,
            public_url,
        })
    }

    /// Get the public URL
    #[allow(dead_code)]
    pub fn public_url(&self) -> &str {
        &self.public_url
    }

    /// Stop the ngrok tunnel
    #[allow(dead_code)]
    pub fn stop(mut self) -> Result<(), NgrokError> {
        self.process
            .kill()
            .map_err(|e| NgrokError::ProcessTerminated(format!("Failed to kill process: {}", e)))?;

        // Wait for process to terminate
        let _ = self.process.wait();

        Ok(())
    }
}

/// Extract public URL from ngrok output line
///
/// Ngrok outputs URLs in the format: https://[random-id].ngrok-free.app
#[allow(dead_code)]
fn extract_public_url(line: &str) -> Option<String> {
    let re = Regex::new(r"https://[a-zA-Z0-9-]+\.ngrok-free\.app").ok()?;
    re.find(line).map(|m| m.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_public_url() {
        let line = "Forwarding https://abc-def-123.ngrok-free.app -> http://localhost:8765";
        let url = extract_public_url(line);
        assert_eq!(url, Some("https://abc-def-123.ngrok-free.app".to_string()));
    }

    #[test]
    fn test_extract_public_url_with_surrounding_text() {
        let line = "Your tunnel is ready at https://my-tunnel-xyz.ngrok-free.app for testing";
        let url = extract_public_url(line);
        assert_eq!(
            url,
            Some("https://my-tunnel-xyz.ngrok-free.app".to_string())
        );
    }

    #[test]
    fn test_extract_public_url_no_match() {
        let line = "Some random log line without URL";
        let url = extract_public_url(line);
        assert_eq!(url, None);
    }

    #[test]
    fn test_extract_public_url_wrong_domain() {
        let line = "https://example.com is not an ngrok URL";
        let url = extract_public_url(line);
        assert_eq!(url, None);
    }
}
