//! Cloudflared tunnel integration for OAuth redirect URI
//!
//! Provides functionality to start cloudflared tunnel, extract public URL,
//! and stop the tunnel after OAuth flow completion.

use regex::Regex;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::Duration;

/// Error type for cloudflared operations
#[derive(Debug)]
#[allow(dead_code)]
pub enum CloudflaredError {
    /// Failed to start cloudflared process
    StartError(String),
    /// Failed to extract public URL from cloudflared output
    UrlExtractionError(String),
    /// Process terminated unexpectedly
    ProcessTerminated(String),
}

impl std::fmt::Display for CloudflaredError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CloudflaredError::StartError(msg) => write!(f, "Failed to start cloudflared: {}", msg),
            CloudflaredError::UrlExtractionError(msg) => {
                write!(f, "Failed to extract URL: {}", msg)
            }
            CloudflaredError::ProcessTerminated(msg) => write!(f, "Process terminated: {}", msg),
        }
    }
}

impl std::error::Error for CloudflaredError {}

/// Cloudflared tunnel manager
pub struct CloudflaredTunnel {
    process: Child,
    public_url: String,
}

impl CloudflaredTunnel {
    /// Start cloudflared tunnel and extract public URL
    ///
    /// # Arguments
    /// * `cloudflared_path` - Path to cloudflared executable (or "cloudflared" to use PATH)
    /// * `local_url` - Local URL to tunnel (e.g., "http://localhost:8765")
    /// * `timeout_secs` - Timeout in seconds to wait for URL extraction
    ///
    /// # Returns
    /// CloudflaredTunnel instance with running process and extracted public URL
    pub fn start(
        cloudflared_path: &str,
        local_url: &str,
        timeout_secs: u64,
    ) -> Result<Self, CloudflaredError> {
        // Start cloudflared process
        let mut process = Command::new(cloudflared_path)
            .args(["tunnel", "--url", local_url])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                CloudflaredError::StartError(format!(
                    "Failed to execute '{}': {}. Make sure cloudflared is installed and accessible.",
                    cloudflared_path, e
                ))
            })?;

        // Extract stdout and stderr
        let stdout = process
            .stdout
            .take()
            .ok_or_else(|| CloudflaredError::StartError("Failed to capture stdout".to_string()))?;
        let stderr = process
            .stderr
            .take()
            .ok_or_else(|| CloudflaredError::StartError("Failed to capture stderr".to_string()))?;

        // Create channel for URL extraction
        let (tx, rx): (std::sync::mpsc::Sender<String>, Receiver<String>) = channel();

        // Spawn thread to read stdout continuously (don't stop after finding URL)
        let tx_clone = tx.clone();
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            let mut url_sent = false;
            for line in reader.lines().map_while(Result::ok) {
                // Send URL once, but keep reading to prevent SIGPIPE
                if !url_sent {
                    if let Some(url) = extract_public_url(&line) {
                        let _ = tx_clone.send(url);
                        url_sent = true;
                    }
                }
                // Continue reading all output to keep pipe open
            }
        });

        // Spawn thread to read stderr continuously (don't stop after finding URL)
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            let mut url_sent = false;
            for line in reader.lines().map_while(Result::ok) {
                // Send URL once, but keep reading to prevent SIGPIPE
                if !url_sent {
                    if let Some(url) = extract_public_url(&line) {
                        let _ = tx.send(url);
                        url_sent = true;
                    }
                }
                // Continue reading all output to keep pipe open
            }
        });

        // Wait for URL with timeout
        let public_url = rx
            .recv_timeout(Duration::from_secs(timeout_secs))
            .map_err(|_| {
                CloudflaredError::UrlExtractionError(format!(
                    "Timeout waiting for cloudflared URL (waited {} seconds). \
                     Make sure cloudflared is working correctly.",
                    timeout_secs
                ))
            })?;

        Ok(Self {
            process,
            public_url,
        })
    }

    /// Get the public URL
    pub fn public_url(&self) -> &str {
        &self.public_url
    }

    /// Check if the tunnel process is still running
    pub fn is_running(&mut self) -> bool {
        match self.process.try_wait() {
            Ok(None) => true, // Process is still running
            Ok(Some(status)) => {
                eprintln!("⚠️  Cloudflared process exited with status: {}", status);
                false
            }
            Err(e) => {
                eprintln!("⚠️  Failed to check cloudflared process status: {}", e);
                false
            }
        }
    }

    /// Stop the cloudflared tunnel
    #[allow(dead_code)]
    pub fn stop(mut self) -> Result<(), CloudflaredError> {
        self.process.kill().map_err(|e| {
            CloudflaredError::ProcessTerminated(format!("Failed to kill process: {}", e))
        })?;

        // Wait for process to terminate
        let _ = self.process.wait();

        Ok(())
    }
}

impl Drop for CloudflaredTunnel {
    fn drop(&mut self) {
        println!("🔴 CloudflaredTunnel is being dropped");
        if self.is_running() {
            println!("  Killing cloudflared process...");
            let _ = self.process.kill();
            let _ = self.process.wait();
        } else {
            println!("  Cloudflared process was already terminated");
        }
    }
}

/// Extract public URL from cloudflared output line
///
/// Cloudflared outputs URLs in the format: https://[random-subdomain].trycloudflare.com
fn extract_public_url(line: &str) -> Option<String> {
    let re = Regex::new(r"https://[a-zA-Z0-9-]+\.trycloudflare\.com").ok()?;
    re.find(line).map(|m| m.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_public_url() {
        let line = "2024-01-01T00:00:00Z INF | https://abc-def-123.trycloudflare.com";
        let url = extract_public_url(line);
        assert_eq!(
            url,
            Some("https://abc-def-123.trycloudflare.com".to_string())
        );
    }

    #[test]
    fn test_extract_public_url_with_surrounding_text() {
        let line = "Your tunnel is ready at https://my-tunnel-xyz.trycloudflare.com for testing";
        let url = extract_public_url(line);
        assert_eq!(
            url,
            Some("https://my-tunnel-xyz.trycloudflare.com".to_string())
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
        let line = "https://example.com is not a cloudflared URL";
        let url = extract_public_url(line);
        assert_eq!(url, None);
    }
}
