//! Integration tests for install-skills command
//!
//! These tests verify the CLI entry point and JSON output format

use serde_json::Value;
use std::process::Command;
use tempfile::TempDir;

/// Helper to run slack-rs binary with args
fn run_slack_rs(args: &[&str]) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_slack-rs"))
        .args(args)
        .output()
        .expect("Failed to execute command");

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    (exit_code, stdout, stderr)
}

#[test]
#[cfg(not(target_os = "windows"))]
fn install_skill_outputs_human_readable_success_message() {
    let temp_dir = TempDir::new().unwrap();
    let temp_home = temp_dir.path();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_slack-rs"));
    cmd.args(["install-skills"])
        .env("HOME", temp_home)
        .current_dir(temp_home);

    let (exit_code, stdout, stderr) = cmd
        .output()
        .map(|output| {
            (
                output.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&output.stdout).to_string(),
                String::from_utf8_lossy(&output.stderr).to_string(),
            )
        })
        .unwrap();

    assert_eq!(exit_code, 0, "Command failed: {}", stderr);
    assert!(
        stdout.contains("Installed 1 skill:"),
        "stdout should show success summary, got: {stdout}"
    );
    assert!(
        stdout.contains("slack-rs ->"),
        "stdout should show installed skill path, got: {stdout}"
    );
}

#[test]
#[cfg(not(target_os = "windows"))]
fn install_skill_json_outputs_required_fields() {
    let temp_dir = TempDir::new().unwrap();
    let temp_home = temp_dir.path();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_slack-rs"));
    cmd.args(["install-skills", "--json"])
        .env("HOME", temp_home)
        .current_dir(temp_home);

    let (exit_code, stdout, stderr) = cmd
        .output()
        .map(|output| {
            (
                output.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&output.stdout).to_string(),
                String::from_utf8_lossy(&output.stderr).to_string(),
            )
        })
        .unwrap();

    assert_eq!(exit_code, 0, "Command failed: {}", stderr);

    let json: Value = serde_json::from_str(&stdout).expect("Invalid JSON output");

    assert_eq!(
        json["schemaVersion"].as_str(),
        Some("1.0"),
        "Missing or incorrect schemaVersion"
    );
    assert_eq!(
        json["type"].as_str(),
        Some("skill-installation"),
        "Missing or incorrect type"
    );
    assert_eq!(json["ok"].as_bool(), Some(true), "Missing or incorrect ok");

    let skills = json["skills"]
        .as_array()
        .expect("skills should be an array");
    assert_eq!(skills.len(), 1, "Should have exactly one skill");

    let skill = &skills[0];
    assert!(skill["name"].is_string(), "skill.name should be a string");
    assert!(skill["path"].is_string(), "skill.path should be a string");
    assert!(
        skill["source_type"].is_string(),
        "skill.source_type should be a string"
    );

    assert_eq!(
        skill["name"].as_str(),
        Some("slack-rs"),
        "Default skill name should be slack-rs"
    );
    assert_eq!(
        skill["source_type"].as_str(),
        Some("self"),
        "Default source_type should be self"
    );
    let path = skill["path"].as_str().expect("skill.path should be string");
    assert!(
        path.ends_with("/.agents/skills/slack-rs") || path.ends_with("\\.agents\\skills\\slack-rs"),
        "Default install should use project .agents path, got: {}",
        path
    );
}

#[test]
fn install_skill_invalid_source_exits_non_zero() {
    let (exit_code, _stdout, stderr) = run_slack_rs(&["install-skills", "github:user/repo"]);

    // Should fail with non-zero exit
    assert_ne!(exit_code, 0, "Should exit non-zero for invalid source");

    // Should have error message on stderr
    assert!(
        stderr.contains("Unknown source scheme") || stderr.contains("Skill installation failed"),
        "stderr should contain error message, got: {}",
        stderr
    );
}

#[test]
fn install_skill_invalid_source_shows_allowed_schemes() {
    let (exit_code, _stdout, stderr) = run_slack_rs(&["install-skills", "foo:bar"]);

    // Should fail with non-zero exit
    assert_ne!(exit_code, 0, "Should exit non-zero for invalid source");

    // Should mention allowed schemes in error message
    assert!(
        stderr.contains("self") && stderr.contains("local:"),
        "Error message should mention allowed schemes (self, local:<path>), got: {}",
        stderr
    );
}

#[test]
fn install_skill_is_routed_from_main() {
    // This test verifies that the command is properly routed
    // We test by running with invalid args to see if it gets to our handler
    let (exit_code, _stdout, stderr) = run_slack_rs(&["install-skills", "invalid:scheme"]);

    // Should fail (because of invalid scheme)
    assert_ne!(exit_code, 0);

    // Should show our error message (not "unknown command")
    assert!(
        !stderr.contains("Slack CLI - Usage"),
        "Should not show main usage (command should be routed)"
    );
}

#[test]
fn commands_json_includes_install_skill() {
    let (exit_code, stdout, _stderr) = run_slack_rs(&["commands", "--json"]);

    assert_eq!(exit_code, 0, "commands --json should succeed");

    let json: Value = serde_json::from_str(&stdout).expect("Invalid JSON output");
    let commands = json["commands"]
        .as_array()
        .expect("commands should be an array");

    // Find install-skills command
    let install_skill_cmd = commands
        .iter()
        .find(|cmd| cmd["name"].as_str() == Some("install-skills"));

    assert!(
        install_skill_cmd.is_some(),
        "install-skills should be in commands list"
    );

    let cmd = install_skill_cmd.unwrap();
    assert!(
        cmd["description"].is_string(),
        "Command should have description"
    );
    assert!(cmd["usage"].is_string(), "Command should have usage");
}

#[test]
fn schema_for_install_skill_is_available() {
    let (exit_code, stdout, _stderr) = run_slack_rs(&[
        "schema",
        "--command",
        "install-skills",
        "--output",
        "json-schema",
    ]);

    assert_eq!(exit_code, 0, "schema command should succeed");

    let json: Value = serde_json::from_str(&stdout).expect("Invalid JSON output");

    // Verify schema response structure
    assert_eq!(json["command"].as_str(), Some("install-skills"));
    assert!(json["schema"].is_object(), "Should have schema object");

    let schema = &json["schema"];

    // Verify schema has required properties
    assert!(
        schema["properties"].is_object(),
        "Schema should have properties"
    );
    assert!(
        schema["properties"]["schemaVersion"].is_object(),
        "Schema should define schemaVersion"
    );
    assert!(
        schema["properties"]["type"].is_object(),
        "Schema should define type"
    );
    assert!(
        schema["properties"]["ok"].is_object(),
        "Schema should define ok"
    );
    assert!(
        schema["properties"]["skills"].is_object(),
        "Schema should define skills"
    );
}

#[test]
fn install_skill_with_local_source() {
    // Create a temporary skill directory
    let skill_dir = TempDir::new().unwrap();
    let skill_path = skill_dir.path();

    // Create minimal skill structure
    std::fs::write(skill_path.join("SKILL.md"), b"# Test Skill").unwrap();

    // Override HOME to use temp directory
    let temp_home = TempDir::new().unwrap();

    let source_arg = format!("local:{}", skill_path.display());
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_slack-rs"));
    cmd.args(["install-skills", &source_arg])
        .current_dir(temp_home.path());

    // Set HOME env on Unix-like systems
    #[cfg(not(target_os = "windows"))]
    cmd.env("HOME", temp_home.path());

    let (exit_code, stdout, stderr) = cmd
        .output()
        .map(|output| {
            (
                output.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&output.stdout).to_string(),
                String::from_utf8_lossy(&output.stderr).to_string(),
            )
        })
        .unwrap();

    assert_eq!(
        exit_code, 0,
        "Install from local should succeed: {}",
        stderr
    );

    // Parse JSON
    assert!(
        stdout.contains("Installed 1 skill:"),
        "stdout should show success summary, got: {stdout}"
    );
    assert!(
        stdout.contains("test skill") || stdout.contains(" -> "),
        "stdout should include installed item, got: {stdout}"
    );
}

#[test]
#[cfg(not(target_os = "windows"))]
fn install_skill_global_uses_home_agents_dir() {
    let temp_home = TempDir::new().unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_slack-rs"));
    cmd.args(["install-skills", "--global"])
        .env("HOME", temp_home.path())
        .current_dir(temp_home.path());

    let (exit_code, stdout, stderr) = cmd
        .output()
        .map(|output| {
            (
                output.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&output.stdout).to_string(),
                String::from_utf8_lossy(&output.stderr).to_string(),
            )
        })
        .unwrap();

    assert_eq!(exit_code, 0, "Global install should succeed: {}", stderr);

    let expected_prefix = temp_home.path().join(".agents").join("skills");
    let expected_prefix_str = expected_prefix.display().to_string();

    assert!(
        stdout.contains("Installed 1 global skill:"),
        "stdout should indicate global install, got: {stdout}"
    );
    assert!(
        stdout.contains(&expected_prefix_str),
        "stdout should include global install path, got: {stdout}"
    );
}
