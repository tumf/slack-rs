//! Clipboard operations with multiple fallback strategies
//!
//! This module provides clipboard copy functionality with a fallback chain:
//! 1. OSC52 (SSH environments with terminal support)
//! 2. arboard (native clipboard via GUI)
//! 3. OS commands (pbcopy/clip/wl-copy/xclip/xsel)

use std::io::Write;
use std::process::Command;

/// Result of a clipboard copy attempt
#[derive(Debug, PartialEq)]
pub enum ClipboardResult {
    /// Copy succeeded with the specified method
    Success(ClipboardMethod),
    /// All methods failed
    Failed,
}

/// Clipboard method used
#[derive(Debug, PartialEq)]
pub enum ClipboardMethod {
    Osc52,
    Arboard,
    OsCommand(&'static str),
}

impl std::fmt::Display for ClipboardMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClipboardMethod::Osc52 => write!(f, "OSC52"),
            ClipboardMethod::Arboard => write!(f, "arboard"),
            ClipboardMethod::OsCommand(cmd) => write!(f, "{}", cmd),
        }
    }
}

/// Try to copy text to clipboard with multiple fallback strategies
///
/// # Arguments
/// * `text` - Text to copy to clipboard
///
/// # Returns
/// * `ClipboardResult` indicating success or failure
pub fn copy_to_clipboard(text: &str) -> ClipboardResult {
    let debug_enabled = std::env::var("SLACK_RS_DEBUG").is_ok();

    // Try OSC52 first if in SSH environment
    if should_try_osc52() {
        if debug_enabled {
            eprintln!("[DEBUG] Trying OSC52...");
        }
        if let Some(osc52_str) = generate_osc52_sequence(text) {
            if try_osc52(&osc52_str).is_ok() {
                if debug_enabled {
                    eprintln!("[DEBUG] OSC52 succeeded");
                }
                return ClipboardResult::Success(ClipboardMethod::Osc52);
            } else if debug_enabled {
                eprintln!("[DEBUG] OSC52 failed");
            }
        } else if debug_enabled {
            eprintln!("[DEBUG] OSC52 not applicable");
        }
    }

    // Try arboard
    if debug_enabled {
        eprintln!("[DEBUG] Trying arboard...");
    }
    if try_arboard(text).is_ok() {
        if debug_enabled {
            eprintln!("[DEBUG] arboard succeeded");
        }
        return ClipboardResult::Success(ClipboardMethod::Arboard);
    } else if debug_enabled {
        eprintln!("[DEBUG] arboard failed");
    }

    // Try OS commands
    if let Some(result) = try_os_commands(text, debug_enabled) {
        return result;
    }

    ClipboardResult::Failed
}

/// Check if we should try OSC52 (TTY + SSH environment)
fn should_try_osc52() -> bool {
    use std::io::IsTerminal;

    // Check if stdout is a TTY
    let is_tty = std::io::stdout().is_terminal();

    // Check for SSH environment variables
    let is_ssh = std::env::var("SSH_CONNECTION").is_ok() || std::env::var("SSH_TTY").is_ok();

    is_tty && is_ssh
}

/// Generate OSC52 escape sequence for clipboard copy
///
/// # Arguments
/// * `text` - Text to encode in OSC52 sequence
///
/// # Returns
/// * `Some(String)` with the OSC52 sequence, or `None` if not applicable
pub fn generate_osc52_sequence(text: &str) -> Option<String> {
    use base64::Engine;

    // Encode text as base64
    let encoded = base64::engine::general_purpose::STANDARD.encode(text);

    // Check if we're in TMUX
    let in_tmux = std::env::var("TMUX").is_ok();

    if in_tmux {
        // TMUX passthrough format: \ePtmux;\e\e]52;c;<base64>\a\e\\
        Some(format!("\x1bPtmux;\x1b\x1b]52;c;{}\x07\x1b\\", encoded))
    } else {
        // Standard OSC52 format: \e]52;c;<base64>\a
        Some(format!("\x1b]52;c;{}\x07", encoded))
    }
}

/// Try to copy using OSC52 sequence
fn try_osc52(osc52_sequence: &str) -> Result<(), std::io::Error> {
    use std::io::IsTerminal;

    // Only write to stdout if it's a terminal
    if !std::io::stdout().is_terminal() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "stdout is not a terminal",
        ));
    }

    let mut stdout = std::io::stdout();
    stdout.write_all(osc52_sequence.as_bytes())?;
    stdout.flush()?;
    Ok(())
}

/// Try to copy using arboard
fn try_arboard(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut clipboard = arboard::Clipboard::new()?;
    clipboard.set_text(text)?;
    Ok(())
}

/// Try OS-specific clipboard commands
fn try_os_commands(text: &str, debug_enabled: bool) -> Option<ClipboardResult> {
    #[cfg(target_os = "macos")]
    {
        if debug_enabled {
            eprintln!("[DEBUG] Trying pbcopy...");
        }
        if try_command_with_stdin("pbcopy", &[], text).is_ok() {
            if debug_enabled {
                eprintln!("[DEBUG] pbcopy succeeded");
            }
            return Some(ClipboardResult::Success(ClipboardMethod::OsCommand(
                "pbcopy",
            )));
        } else if debug_enabled {
            eprintln!("[DEBUG] pbcopy failed");
        }
    }

    #[cfg(target_os = "windows")]
    {
        if debug_enabled {
            eprintln!("[DEBUG] Trying clip...");
        }
        if try_command_with_stdin("cmd", &["/C", "clip"], text).is_ok() {
            if debug_enabled {
                eprintln!("[DEBUG] clip succeeded");
            }
            return Some(ClipboardResult::Success(ClipboardMethod::OsCommand("clip")));
        } else if debug_enabled {
            eprintln!("[DEBUG] clip failed");
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Try wl-copy (Wayland)
        if debug_enabled {
            eprintln!("[DEBUG] Trying wl-copy...");
        }
        if command_exists("wl-copy") {
            if try_command_with_stdin("wl-copy", &[], text).is_ok() {
                if debug_enabled {
                    eprintln!("[DEBUG] wl-copy succeeded");
                }
                return Some(ClipboardResult::Success(ClipboardMethod::OsCommand(
                    "wl-copy",
                )));
            } else if debug_enabled {
                eprintln!("[DEBUG] wl-copy failed");
            }
        } else if debug_enabled {
            eprintln!("[DEBUG] wl-copy not found");
        }

        // Try xclip (X11)
        if debug_enabled {
            eprintln!("[DEBUG] Trying xclip...");
        }
        if command_exists("xclip") {
            if try_command_with_stdin("xclip", &["-selection", "clipboard"], text).is_ok() {
                if debug_enabled {
                    eprintln!("[DEBUG] xclip succeeded");
                }
                return Some(ClipboardResult::Success(ClipboardMethod::OsCommand(
                    "xclip",
                )));
            } else if debug_enabled {
                eprintln!("[DEBUG] xclip failed");
            }
        } else if debug_enabled {
            eprintln!("[DEBUG] xclip not found");
        }

        // Try xsel (X11)
        if debug_enabled {
            eprintln!("[DEBUG] Trying xsel...");
        }
        if command_exists("xsel") {
            if try_command_with_stdin("xsel", &["--clipboard", "--input"], text).is_ok() {
                if debug_enabled {
                    eprintln!("[DEBUG] xsel succeeded");
                }
                return Some(ClipboardResult::Success(ClipboardMethod::OsCommand("xsel")));
            } else if debug_enabled {
                eprintln!("[DEBUG] xsel failed");
            }
        } else if debug_enabled {
            eprintln!("[DEBUG] xsel not found");
        }
    }

    None
}

/// Check if a command exists in PATH
#[cfg(target_os = "linux")]
fn command_exists(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .output()
        .or_else(|_| Command::new(cmd).arg("-v").output())
        .is_ok()
}

/// Try to run a command with stdin input
fn try_command_with_stdin(
    cmd: &str,
    args: &[&str],
    input: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes())?;
        stdin.flush()?;
        drop(stdin);
    }

    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("Command failed with status: {}", status).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[serial_test::serial]
    fn test_generate_osc52_sequence_without_tmux() {
        // Clear TMUX env var for this test
        let original = std::env::var("TMUX").ok();
        std::env::remove_var("TMUX");

        let text = "Hello, World!";
        let result = generate_osc52_sequence(text);
        assert!(result.is_some());

        let sequence = result.unwrap();
        // Should contain base64-encoded text
        assert!(sequence.contains("SGVsbG8sIFdvcmxkIQ=="));
        // Should use standard OSC52 format
        assert!(sequence.starts_with("\x1b]52;c;"));
        assert!(sequence.ends_with("\x07"));
        // Should NOT contain TMUX passthrough
        assert!(!sequence.contains("Ptmux"));

        // Restore original TMUX env var
        if let Some(val) = original {
            std::env::set_var("TMUX", val);
        }
    }

    #[test]
    #[serial_test::serial]
    fn test_generate_osc52_sequence_with_tmux() {
        // Set TMUX env var for this test
        let original = std::env::var("TMUX").ok();
        std::env::set_var("TMUX", "/tmp/tmux-1000/default,12345,0");

        let text = "Hello, World!";
        let result = generate_osc52_sequence(text);
        assert!(result.is_some());

        let sequence = result.unwrap();
        // Should contain base64-encoded text
        assert!(sequence.contains("SGVsbG8sIFdvcmxkIQ=="));
        // Should use TMUX passthrough format
        assert!(sequence.starts_with("\x1bPtmux;"));
        assert!(sequence.ends_with("\x1b\\"));
        assert!(sequence.contains("\x1b\x1b]52;c;"));

        // Restore original TMUX env var
        match original {
            Some(val) => std::env::set_var("TMUX", val),
            None => std::env::remove_var("TMUX"),
        }
    }

    #[test]
    fn test_should_try_osc52_requires_ssh_env() {
        // This test documents the behavior but can't reliably test it
        // since it depends on actual terminal and environment state
        // Just verify the function doesn't panic
        let _ = should_try_osc52();
    }

    #[test]
    fn test_clipboard_method_display() {
        assert_eq!(format!("{}", ClipboardMethod::Osc52), "OSC52");
        assert_eq!(format!("{}", ClipboardMethod::Arboard), "arboard");
        assert_eq!(
            format!("{}", ClipboardMethod::OsCommand("pbcopy")),
            "pbcopy"
        );
    }

    #[test]
    fn test_clipboard_result_enum() {
        // Test that ClipboardResult variants work as expected
        let success = ClipboardResult::Success(ClipboardMethod::Arboard);
        let failed = ClipboardResult::Failed;

        assert!(matches!(success, ClipboardResult::Success(_)));
        assert!(matches!(failed, ClipboardResult::Failed));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_command_exists() {
        // Test with a command that should always exist
        assert!(command_exists("ls") || command_exists("echo"));

        // Test with a command that should not exist
        assert!(!command_exists(
            "this-command-definitely-does-not-exist-12345"
        ));
    }

    #[test]
    #[serial_test::serial]
    fn test_osc52_sequence_format() {
        // Test the exact format of OSC52 sequences
        let original_tmux = std::env::var("TMUX").ok();
        std::env::remove_var("TMUX");

        let seq = generate_osc52_sequence("test").unwrap();
        // Standard format: ESC ] 52 ; c ; <base64> BEL
        assert!(seq.starts_with("\x1b]52;c;"));
        assert!(seq.ends_with("\x07"));

        std::env::set_var("TMUX", "dummy");
        let seq_tmux = generate_osc52_sequence("test").unwrap();
        // TMUX format: ESC P tmux ; ESC ESC ] 52 ; c ; <base64> BEL ESC \
        assert!(seq_tmux.starts_with("\x1bPtmux;"));
        assert!(seq_tmux.contains("\x1b\x1b]52;c;"));
        assert!(seq_tmux.ends_with("\x1b\\"));

        // Restore
        match original_tmux {
            Some(val) => std::env::set_var("TMUX", val),
            None => std::env::remove_var("TMUX"),
        }
    }
}
