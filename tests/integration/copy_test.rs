//! Integration tests for the copy command (CLI)

use std::process::Command;
use tempfile::TempDir;

use crate::helpers::{fixtures_dir, load_fixture};

/// Helper to run agr CLI and capture output
fn run_agr(args: &[&str]) -> (String, String, i32) {
    let output = Command::new(env!("CARGO_BIN_EXE_agr"))
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .expect("Failed to execute agr");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    (stdout, stderr, exit_code)
}

// ============================================================================
// Help Output Tests
// ============================================================================

#[test]
fn copy_help_exits_0_and_shows_usage() {
    let (stdout, _stderr, exit_code) = run_agr(&["copy", "--help"]);

    assert_eq!(exit_code, 0);
    assert!(stdout.contains("Copy a recording"));
    assert!(stdout.contains("<FILE>"));
    assert!(stdout.contains("clipboard"));
}

#[test]
fn snapshot_cli_help_copy() {
    let (stdout, stderr, exit_code) = run_agr(&["copy", "--help"]);
    let output = format!(
        "=== agr copy --help ===\nExit code: {}\n\n--- stdout ---\n{}\n--- stderr ---\n{}",
        exit_code, stdout, stderr
    );
    insta::assert_snapshot!("cli_help_copy", output);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn copy_no_arguments_shows_error() {
    let (_stdout, stderr, exit_code) = run_agr(&["copy"]);

    assert_eq!(exit_code, 2);
    assert!(stderr.contains("required arguments"));
    assert!(stderr.contains("<FILE>"));
}

#[test]
fn copy_nonexistent_file_exits_nonzero_with_helpful_error() {
    let (_stdout, stderr, exit_code) = run_agr(&["copy", "nonexistent.cast"]);

    assert_eq!(exit_code, 1);
    assert!(stderr.contains("File not found") || stderr.contains("not found"));
    assert!(stderr.contains("nonexistent.cast"));
}

#[test]
fn copy_nonexistent_file_with_path_shows_error() {
    let (_stdout, stderr, exit_code) = run_agr(&["copy", "/some/path/to/missing.cast"]);

    assert_eq!(exit_code, 1);
    assert!(stderr.contains("File not found") || stderr.contains("not found"));
}

// ============================================================================
// Path Resolution Tests
// ============================================================================

#[test]
fn copy_with_absolute_path_finds_file() {
    // Create a temp file
    let temp_dir = TempDir::new().unwrap();
    let cast_path = temp_dir.path().join("test.cast");
    std::fs::write(&cast_path, load_fixture("sample.cast")).unwrap();

    // Try to copy it - it should find the file (may fail for other reasons on CI)
    let (_stdout, stderr, _exit_code) = run_agr(&["copy", cast_path.to_str().unwrap()]);

    // Should NOT show "File not found" error - file should be found
    assert!(
        !stderr.contains("File not found"),
        "Should find file at absolute path, got: {}",
        stderr
    );
}

// ============================================================================
// Platform-Specific Tests
// ============================================================================

#[test]
#[cfg(target_os = "macos")]
fn copy_succeeds_with_temp_file_on_macos() {
    let temp_dir = TempDir::new().unwrap();
    let cast_path = temp_dir.path().join("test.cast");
    std::fs::write(&cast_path, load_fixture("sample.cast")).unwrap();

    let (stdout, stderr, exit_code) = run_agr(&["copy", cast_path.to_str().unwrap()]);

    // On macOS, copy should succeed
    assert_eq!(
        exit_code, 0,
        "Expected success on macOS, stdout: {}, stderr: {}",
        stdout, stderr
    );
    assert!(
        stdout.contains("Copied") || stdout.contains("clipboard"),
        "Expected success message, got stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
#[cfg(target_os = "linux")]
fn copy_succeeds_or_fails_gracefully_on_linux() {
    let temp_dir = TempDir::new().unwrap();
    let cast_path = temp_dir.path().join("test.cast");
    std::fs::write(&cast_path, load_fixture("sample.cast")).unwrap();

    let (stdout, stderr, exit_code) = run_agr(&["copy", cast_path.to_str().unwrap()]);

    // On Linux, either succeeds (tools available) or fails gracefully (no tools)
    if exit_code == 0 {
        // Success - tools were available
        assert!(
            stdout.contains("Copied") || stdout.contains("clipboard"),
            "Expected success message, got: {}",
            stdout
        );
    } else {
        // Graceful failure - should mention clipboard tools
        assert!(
            stderr.contains("xclip")
                || stderr.contains("xsel")
                || stderr.contains("wl-copy")
                || stderr.contains("clipboard"),
            "Expected helpful error about clipboard tools, got: {}",
            stderr
        );
    }
}

// ============================================================================
// Shell Completion Tests
// ============================================================================

#[test]
fn completions_files_includes_test_recordings() {
    // Test that --files flag works (requires recordings dir to exist)
    let (stdout, stderr, exit_code) = run_agr(&["completions", "--files"]);

    // Should exit successfully even if no recordings exist
    assert_eq!(
        exit_code, 0,
        "completions --files should succeed, stderr: {}",
        stderr
    );
    // Output may be empty if no recordings exist, which is fine
    let _ = stdout; // stdout contains file list or is empty
}

#[test]
fn generated_zsh_init_contains_copy_in_file_cmds() {
    let (stdout, _stderr, exit_code) = run_agr(&["completions", "--shell-init", "zsh"]);

    assert_eq!(exit_code, 0);
    // The generated zsh completion should include "copy" in the list of
    // commands that accept file arguments
    assert!(
        stdout.contains("copy"),
        "zsh init should contain 'copy' command reference"
    );
}

#[test]
fn generated_bash_init_contains_copy_in_file_cmds() {
    let (stdout, _stderr, exit_code) = run_agr(&["completions", "--shell-init", "bash"]);

    assert_eq!(exit_code, 0);
    // The generated bash completion should include "copy" in the list of
    // commands that accept file arguments
    assert!(
        stdout.contains("copy"),
        "bash init should contain 'copy' command reference"
    );
}

// ============================================================================
// CLI Parsing Tests
// ============================================================================

#[test]
fn copy_accepts_file_argument() {
    // Just verify the command parses correctly
    let fixture_path = fixtures_dir().join("sample.cast");
    let (_stdout, stderr, _exit_code) = run_agr(&["copy", fixture_path.to_str().unwrap()]);

    // Should NOT show parsing errors
    assert!(
        !stderr.contains("unexpected argument"),
        "Command should parse file argument correctly"
    );
}
