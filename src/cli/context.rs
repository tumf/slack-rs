//! CLI execution context with non-interactive mode support
use std::io::IsTerminal;

/// CLI execution context
///
/// Tracks global CLI state such as non-interactive mode
#[derive(Debug, Clone)]
pub struct CliContext {
    /// Whether to run in non-interactive mode (no prompts)
    pub non_interactive: bool,
}

impl CliContext {
    /// Create a new CLI context with auto-detection of TTY
    ///
    /// # Arguments
    /// * `explicit_non_interactive` - Explicit --non-interactive flag from CLI
    ///
    /// # Behavior
    /// - If `explicit_non_interactive` is true, force non-interactive mode
    /// - Otherwise, auto-detect based on stdin TTY status
    /// - When stdin is not a TTY, automatically enable non-interactive mode
    pub fn new(explicit_non_interactive: bool) -> Self {
        let non_interactive = explicit_non_interactive || !std::io::stdin().is_terminal();
        Self { non_interactive }
    }

    /// Check if running in non-interactive mode
    pub fn is_non_interactive(&self) -> bool {
        self.non_interactive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explicit_non_interactive() {
        let ctx = CliContext::new(true);
        assert!(ctx.is_non_interactive());
    }

    #[test]
    fn test_context_clone() {
        let ctx = CliContext::new(true);
        let cloned = ctx.clone();
        assert_eq!(ctx.non_interactive, cloned.non_interactive);
    }
}
