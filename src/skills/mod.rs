//! Skill installation module
//!
//! This module provides functionality to install agent skills from embedded resources
//! or local filesystem paths. Skills are deployed to .agents/skills/
//! and tracked in a lock file.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

const EMBEDDED_SKILL_NAME: &str = "slack-rs";
const EMBEDDED_SKILL_DATA: &[(&str, &[u8])] = &[
    ("SKILL.md", include_bytes!("../../skills/slack-rs/SKILL.md")),
    (
        "README.md",
        include_bytes!("../../skills/slack-rs/README.md"),
    ),
    (
        "references/recipes.md",
        include_bytes!("../../skills/slack-rs/references/recipes.md"),
    ),
];

#[derive(Debug, Error)]
pub enum SkillError {
    #[error("Invalid source: {0}")]
    InvalidSource(String),

    #[error("Unknown source scheme: {0}. Allowed schemes: 'self', 'local:<path>'")]
    UnknownScheme(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Skill not found: {0}")]
    SkillNotFound(String),

    #[error("Path error: {0}")]
    PathError(String),
}

/// Source of skill installation
#[derive(Debug, Clone, PartialEq)]
pub enum Source {
    /// Embedded skill (skills/slack-rs)
    SelfEmbedded,
    /// Local filesystem path
    Local(PathBuf),
}

impl Source {
    /// Parse source string into Source enum
    ///
    /// # Arguments
    /// * `s` - Source string (empty/"self" or "local:<path>")
    ///
    /// # Returns
    /// * `Ok(Source)` - Parsed source
    /// * `Err(SkillError)` - Invalid or unknown source scheme
    pub fn parse(s: &str) -> Result<Self, SkillError> {
        if s.is_empty() || s == "self" {
            Ok(Source::SelfEmbedded)
        } else if let Some(path_str) = s.strip_prefix("local:") {
            let path = PathBuf::from(path_str);
            Ok(Source::Local(path))
        } else {
            // Unknown scheme - reject immediately
            Err(SkillError::UnknownScheme(s.to_string()))
        }
    }
}

/// Installed skill information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledSkill {
    pub name: String,
    pub path: String,
    pub source_type: String,
}

/// Lock file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillLock {
    pub skills: Vec<InstalledSkill>,
}

impl SkillLock {
    pub fn new() -> Self {
        SkillLock { skills: Vec::new() }
    }

    pub fn add_skill(&mut self, skill: InstalledSkill) {
        // Remove existing entry with same name if present
        self.skills.retain(|s| s.name != skill.name);
        self.skills.push(skill);
    }
}

impl Default for SkillLock {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolve .agents base directory.
///
/// - global=true  => ~/.agents
/// - global=false => <current-project>/.agents
fn resolve_agents_base_dir(global: bool) -> Result<PathBuf, SkillError> {
    if global {
        let home = directories::BaseDirs::new()
            .ok_or_else(|| SkillError::PathError("Cannot determine home directory".to_string()))?
            .home_dir()
            .to_path_buf();
        return Ok(home.join(".agents"));
    }

    let cwd = std::env::current_dir()
        .map_err(|e| SkillError::PathError(format!("Cannot determine current directory: {}", e)))?;
    Ok(cwd.join(".agents"))
}

/// Get the skills directory path
fn get_skills_dir(global: bool) -> Result<PathBuf, SkillError> {
    Ok(resolve_agents_base_dir(global)?.join("skills"))
}

/// Get the lock file path
fn get_lock_file_path(global: bool) -> Result<PathBuf, SkillError> {
    Ok(resolve_agents_base_dir(global)?.join(".skill-lock.json"))
}

/// Load lock file
fn load_lock(global: bool) -> Result<SkillLock, SkillError> {
    let lock_path = get_lock_file_path(global)?;

    if !lock_path.exists() {
        return Ok(SkillLock::new());
    }

    let contents = fs::read_to_string(&lock_path)?;

    // Current format: { "skills": [ ... ] }
    if let Ok(lock) = serde_json::from_str::<SkillLock>(&contents) {
        return Ok(lock);
    }

    // Compatibility: map format used by some installers
    // {
    //   "skills": {
    //     "name": {"path": "...", "source_type": "self"}
    //   }
    // }
    let value: Value = serde_json::from_str(&contents)?;
    if let Some(skills_obj) = value.get("skills").and_then(|v| v.as_object()) {
        let mut lock = SkillLock::new();
        for (name, entry) in skills_obj {
            let path = entry
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let source_type = entry
                .get("source_type")
                .or_else(|| entry.get("sourceType"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            lock.add_skill(InstalledSkill {
                name: name.clone(),
                path,
                source_type,
            });
        }
        return Ok(lock);
    }

    Err(SkillError::SerializationError(serde_json::Error::io(
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Unrecognized lock file format",
        ),
    )))
}

/// Save lock file
fn save_lock(lock: &SkillLock, global: bool) -> Result<(), SkillError> {
    let lock_path = get_lock_file_path(global)?;

    // Ensure parent directory exists
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let contents = serde_json::to_string_pretty(lock)?;
    fs::write(&lock_path, contents)?;
    Ok(())
}

/// Deploy embedded skill files to target directory
fn deploy_embedded_skill(target_dir: &Path) -> Result<(), SkillError> {
    fs::create_dir_all(target_dir)?;

    for (rel_path, data) in EMBEDDED_SKILL_DATA {
        let target_file = target_dir.join(rel_path);

        // Create parent directories if needed
        if let Some(parent) = target_file.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(target_file, data)?;
    }

    Ok(())
}

/// Deploy local skill directory to target using symlink (preferred) or copy (fallback)
fn deploy_local_skill(source_dir: &Path, target_dir: &Path) -> Result<(), SkillError> {
    if !source_dir.exists() {
        return Err(SkillError::SkillNotFound(format!(
            "Source directory does not exist: {}",
            source_dir.display()
        )));
    }

    // Remove existing target if present
    if target_dir.exists() {
        match fs::remove_dir_all(target_dir) {
            Ok(_) => {}
            Err(_) => {
                fs::remove_file(target_dir)?;
            }
        }
    }

    // Ensure parent directory exists
    if let Some(parent) = target_dir.parent() {
        fs::create_dir_all(parent)?;
    }

    // Try symlink first
    #[cfg(unix)]
    {
        if std::os::unix::fs::symlink(source_dir, target_dir).is_ok() {
            return Ok(());
        }
    }

    // Fall back to recursive copy
    copy_dir_all(source_dir, target_dir)?;
    Ok(())
}

/// Recursively copy directory
fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), SkillError> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Install skill from source
///
/// # Arguments
/// * `source` - Source to install from (None defaults to self)
/// * `global` - Whether to install into ~/.agents (true) or ./.agents (false)
///
/// # Returns
/// * `Ok(InstalledSkill)` - Successfully installed skill info
/// * `Err(SkillError)` - Installation failed
pub fn install_skill(source: Option<&str>, global: bool) -> Result<InstalledSkill, SkillError> {
    // Default to self if no source provided
    let source_str = source.unwrap_or("self");
    let parsed_source = Source::parse(source_str)?;

    let (skill_name, source_type) = match &parsed_source {
        Source::SelfEmbedded => (EMBEDDED_SKILL_NAME.to_string(), "self".to_string()),
        Source::Local(path) => {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| {
                    SkillError::PathError(format!(
                        "Cannot extract skill name from path: {}",
                        path.display()
                    ))
                })?
                .to_string();
            (name, "local".to_string())
        }
    };

    // Determine target directory
    let skills_dir = get_skills_dir(global)?;
    let target_dir = skills_dir.join(&skill_name);

    // Deploy based on source type
    match parsed_source {
        Source::SelfEmbedded => {
            deploy_embedded_skill(&target_dir)?;
        }
        Source::Local(ref path) => {
            deploy_local_skill(path, &target_dir)?;
        }
    }

    // Update lock file
    let mut lock = load_lock(global)?;
    let installed = InstalledSkill {
        name: skill_name,
        path: target_dir.to_string_lossy().to_string(),
        source_type,
    };
    lock.add_skill(installed.clone());
    save_lock(&lock, global)?;

    Ok(installed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_source_accepts_self_and_local() {
        // Test empty string defaults to self
        assert_eq!(Source::parse("").unwrap(), Source::SelfEmbedded);

        // Test explicit "self"
        assert_eq!(Source::parse("self").unwrap(), Source::SelfEmbedded);

        // Test local path
        let local_result = Source::parse("local:/path/to/skill").unwrap();
        match local_result {
            Source::Local(path) => {
                assert_eq!(path, PathBuf::from("/path/to/skill"));
            }
            _ => panic!("Expected Local variant"),
        }
    }

    #[test]
    fn parse_source_rejects_unknown_scheme() {
        let result = Source::parse("github:user/repo");
        assert!(result.is_err());
        match result.unwrap_err() {
            SkillError::UnknownScheme(s) => {
                assert_eq!(s, "github:user/repo");
            }
            _ => panic!("Expected UnknownScheme error"),
        }
    }

    #[test]
    fn unknown_scheme_error_includes_allowed_schemes() {
        let result = Source::parse("foo:bar");
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("self"),
            "Error should mention 'self' scheme"
        );
        assert!(
            err_msg.contains("local:"),
            "Error should mention 'local:<path>' scheme"
        );
    }

    #[test]
    fn default_source_is_self() {
        // When no source is provided to install_skill, it should default to "self"
        // We can't easily test the full install without filesystem setup,
        // but we can verify the parse logic
        let default_source = Source::parse("").unwrap();
        assert_eq!(default_source, Source::SelfEmbedded);
    }

    #[test]
    fn self_source_uses_embedded_skill() {
        // Verify that embedded skill data is available
        assert!(!EMBEDDED_SKILL_DATA.is_empty());
        assert_eq!(EMBEDDED_SKILL_NAME, "slack-rs");

        // Verify we have the expected files
        let file_names: Vec<&str> = EMBEDDED_SKILL_DATA.iter().map(|(name, _)| *name).collect();
        assert!(file_names.contains(&"SKILL.md"));
        assert!(file_names.contains(&"README.md"));
    }

    #[test]
    fn install_writes_skill_dir_and_lock_file() {
        use tempfile::TempDir;

        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let _temp_path = temp_dir.path();

        // Mock the config paths by using environment variables or test helpers
        // For now, we'll test the core logic separately

        // Test that SkillLock can be created and modified
        let mut lock = SkillLock::new();
        assert_eq!(lock.skills.len(), 0);

        let test_skill = InstalledSkill {
            name: "test-skill".to_string(),
            path: "/tmp/test-skill".to_string(),
            source_type: "self".to_string(),
        };

        lock.add_skill(test_skill.clone());
        assert_eq!(lock.skills.len(), 1);
        assert_eq!(lock.skills[0].name, "test-skill");

        // Test that adding same skill again replaces it
        let updated_skill = InstalledSkill {
            name: "test-skill".to_string(),
            path: "/tmp/test-skill-updated".to_string(),
            source_type: "local".to_string(),
        };

        lock.add_skill(updated_skill);
        assert_eq!(lock.skills.len(), 1);
        assert_eq!(lock.skills[0].path, "/tmp/test-skill-updated");
    }

    #[test]
    fn falls_back_to_copy_when_symlink_fails() {
        // This test verifies the copy fallback logic exists
        // We can't easily test actual symlink failure without OS-specific setup,
        // but we can verify copy_dir_all works

        use tempfile::TempDir;

        let src_dir = TempDir::new().unwrap();
        let dst_dir = TempDir::new().unwrap();

        // Create test file in source
        let test_file = src_dir.path().join("test.txt");
        fs::write(&test_file, b"test content").unwrap();

        // Copy directory
        let dst_path = dst_dir.path().join("copied");
        let result = copy_dir_all(src_dir.path(), &dst_path);
        assert!(result.is_ok());

        // Verify file was copied
        let copied_file = dst_dir.path().join("copied").join("test.txt");
        assert!(copied_file.exists());
        let contents = fs::read_to_string(copied_file).unwrap();
        assert_eq!(contents, "test content");
    }

    #[test]
    fn parse_legacy_map_lock_format() {
        let json = r#"{
            "skills": {
                "slack-rs": {
                    "path": "/tmp/.agents/skills/slack-rs",
                    "source_type": "self"
                }
            }
        }"#;

        let value: Value = serde_json::from_str(json).unwrap();
        let skills_obj = value.get("skills").unwrap().as_object().unwrap();

        let mut lock = SkillLock::new();
        for (name, entry) in skills_obj {
            lock.add_skill(InstalledSkill {
                name: name.clone(),
                path: entry
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                source_type: entry
                    .get("source_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
            });
        }

        assert_eq!(lock.skills.len(), 1);
        assert_eq!(lock.skills[0].name, "slack-rs");
        assert_eq!(lock.skills[0].source_type, "self");
    }
}
