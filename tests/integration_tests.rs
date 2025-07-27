// Comprehensive integration tests for tree functionality
// Tests all major features with actual binary execution

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn cmd() -> Command {
    Command::cargo_bin("tree").unwrap()
}

fn create_test_structure() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create test directory structure
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::create_dir_all(root.join(".git")).unwrap();

    fs::write(root.join("README.md"), "# Test").unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(root.join("src/lib.rs"), "// lib").unwrap();
    fs::write(root.join("tests/test.rs"), "// test").unwrap();
    fs::write(root.join("docs/guide.md"), "# Guide").unwrap();
    fs::write(root.join(".gitignore"), "target/").unwrap();

    temp_dir
}

fn output_contains(output: &[u8], text: &str) -> bool {
    String::from_utf8_lossy(output).contains(text)
}

fn output_not_contains(output: &[u8], text: &str) -> bool {
    !output_contains(output, text)
}

#[test]
fn test_basic_directory_listing() {
    let temp_dir = create_test_structure();

    let output = cmd()
        .arg(temp_dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert!(output_contains(&output, "README.md"));
    assert!(output_contains(&output, "src"));
    assert!(output_contains(&output, "tests"));
    assert!(output_contains(&output, "docs"));
}

#[test]
fn test_hidden_files() {
    let temp_dir = create_test_structure();

    // Without -a flag
    let output_no_hidden = cmd()
        .arg(temp_dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert!(output_not_contains(&output_no_hidden, ".git"));
    assert!(output_not_contains(&output_no_hidden, ".gitignore"));

    // With -a flag
    let output_with_hidden = cmd()
        .args(["-a", temp_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert!(output_contains(&output_with_hidden, ".git"));
    assert!(output_contains(&output_with_hidden, ".gitignore"));
}

#[test]
fn test_depth_control() {
    let temp_dir = create_test_structure();

    // Level 1 - should see directories but not their contents
    let output_l1 = cmd()
        .args(["-L", "1", temp_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert!(output_contains(&output_l1, "src"));
    assert!(output_not_contains(&output_l1, "main.rs"));

    // Level 2 - should see file contents
    let output_l2 = cmd()
        .args(["-L", "2", temp_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert!(output_contains(&output_l2, "src"));
    assert!(output_contains(&output_l2, "main.rs"));
}

#[test]
fn test_directories_only() {
    let temp_dir = create_test_structure();

    let output = cmd()
        .args(["-d", temp_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert!(output_contains(&output, "src"));
    assert!(output_contains(&output, "tests"));
    assert!(output_contains(&output, "docs"));
    assert!(output_not_contains(&output, "README.md"));
    assert!(output_not_contains(&output, "main.rs"));
}

#[test]
fn test_file_sizes() {
    let temp_dir = create_test_structure();

    // Size in bytes
    let output_bytes = cmd()
        .args(["-s", temp_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Should contain file sizes (exact numbers may vary)
    let output_str = String::from_utf8_lossy(&output_bytes);
    assert!(output_str.chars().any(|c| c.is_ascii_digit())); // Should contain some numbers

    // Human readable sizes
    let output_human = cmd()
        .args(["-H", temp_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert!(output_contains(&output_human, "B")); // Should show bytes unit
}

#[test]
fn test_pattern_matching() {
    let temp_dir = create_test_structure();

    // Include pattern
    let output_include = cmd()
        .args(["-P", "*.md", temp_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert!(output_contains(&output_include, "README.md"));
    assert!(output_contains(&output_include, "guide.md"));
    assert!(output_not_contains(&output_include, "main.rs"));

    // Exclude pattern
    let output_exclude = cmd()
        .args(["-I", "*.rs", temp_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert!(output_contains(&output_exclude, "README.md"));
    assert!(output_not_contains(&output_exclude, "main.rs"));
    assert!(output_not_contains(&output_exclude, "test.rs"));
}

#[test]
fn test_ascii_mode() {
    let temp_dir = create_test_structure();

    let output = cmd()
        .args(["-A", temp_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // ASCII mode should use | and ` instead of Unicode box chars
    let output_str = String::from_utf8_lossy(&output);
    assert!(output_str.contains("|") || output_str.contains("`") || output_str.contains("+"));
}

#[test]
fn test_color_options() {
    let temp_dir = create_test_structure();

    // Force color on
    cmd()
        .args(["-C", temp_dir.path().to_str().unwrap()])
        .assert()
        .success();

    // Force color off
    cmd()
        .args(["-n", temp_dir.path().to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn test_full_path() {
    let temp_dir = create_test_structure();

    let output = cmd()
        .args(["-f", temp_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert!(output_contains(&output, temp_dir.path().to_str().unwrap()));
}

#[test]
fn test_sort_options() {
    let temp_dir = create_test_structure();

    // Sort by time
    cmd()
        .args(["-t", temp_dir.path().to_str().unwrap()])
        .assert()
        .success();

    // Reverse sort
    cmd()
        .args(["-r", temp_dir.path().to_str().unwrap()])
        .assert()
        .success();

    // Both together
    cmd()
        .args(["-tr", temp_dir.path().to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn test_modification_date() {
    let temp_dir = create_test_structure();

    let output = cmd()
        .args(["-D", temp_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert!(output_contains(&output, "202")); // Should contain year
}

#[test]
fn test_classify() {
    let temp_dir = create_test_structure();

    let output = cmd()
        .args(["-F", temp_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert!(output_contains(&output, "/")); // Directories should have trailing /
}

#[test]
fn test_no_report() {
    let temp_dir = create_test_structure();

    // With report (default)
    let with_report = cmd()
        .arg(temp_dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Without report
    let without_report = cmd()
        .args(["--noreport", temp_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let with_str = String::from_utf8_lossy(&with_report);
    let without_str = String::from_utf8_lossy(&without_report);

    // With report should contain summary
    assert!(with_str.contains("directories") || with_str.contains("files"));
    // Without report should not contain summary numbers
    assert!(without_str.len() < with_str.len());
}

#[test]
fn test_output_to_file() {
    let temp_dir = create_test_structure();
    let output_file = temp_dir.path().join("output.txt");

    cmd()
        .args([
            "-o",
            output_file.to_str().unwrap(),
            temp_dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    // Check that output file was created and contains expected content
    assert!(output_file.exists());
    let content = fs::read_to_string(&output_file).unwrap();
    assert!(content.contains("README.md"));
    assert!(content.contains("src"));
}

#[test]
fn test_fromfile_basic() {
    // Test basic fromfile functionality with simple paths
    let simple_paths = "src/\nsrc/main.rs\nsrc/lib.rs\ntests/\ntests/test.rs\n";

    let output = cmd()
        .arg("--fromfile")
        .arg(".")
        .write_stdin(simple_paths)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert!(output_contains(&output, "src"));
    assert!(output_contains(&output, "main.rs"));
    assert!(output_contains(&output, "tests"));
}

#[test]
fn test_fromfile_windows_paths() {
    // Test cross-platform path normalization
    let windows_paths = "src\\\nsrc\\main.rs\nsrc\\lib.rs\ntests\\\ntests\\test.rs\n";

    let output = cmd()
        .arg("--fromfile")
        .arg(".")
        .write_stdin(windows_paths)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert!(output_contains(&output, "src"));
    assert!(output_contains(&output, "main.rs"));
    assert!(output_contains(&output, "tests"));
}

#[test]
fn test_fromfile_with_flags() {
    let simple_paths = "src/\nsrc/main.rs\nsrc/lib.rs\nREADME.md\n";

    // Test fromfile with directories only
    let output_dirs = cmd()
        .args(["--fromfile", "-d", "."])
        .write_stdin(simple_paths)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert!(output_contains(&output_dirs, "src"));
    assert!(output_not_contains(&output_dirs, "main.rs"));
}

#[test]
fn test_pattern_filtering_prunes_empty_directories() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create test structure with empty and non-empty directories
    fs::create_dir_all(root.join("empty_branch/empty_sub")).unwrap();
    fs::create_dir_all(root.join("match_branch")).unwrap();
    fs::create_dir_all(root.join("partial_match/has_files")).unwrap();
    fs::create_dir_all(root.join("partial_match/empty_sub")).unwrap();
    
    // Create files
    fs::write(root.join("match_branch/file.txt"), "content").unwrap();
    fs::write(root.join("partial_match/has_files/test.txt"), "content").unwrap();
    fs::write(root.join("other.log"), "content").unwrap();

    // Test pattern filtering
    let output = cmd()
        .args(["-P", "*.txt", temp_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Should show directories that contain matching files
    assert!(output_contains(&output, "match_branch"));
    assert!(output_contains(&output, "partial_match"));
    assert!(output_contains(&output, "has_files"));
    assert!(output_contains(&output, "file.txt"));
    assert!(output_contains(&output, "test.txt"));

    // Should NOT show empty directories
    assert!(output_not_contains(&output, "empty_branch"));
    assert!(output_not_contains(&output, "empty_sub"));
    assert!(output_not_contains(&output, "other.log"));
}

#[test]
fn test_error_handling() {
    // Test with non-existent directory - should succeed but with error in stderr
    let nonexistent_path = if cfg!(windows) {
        "C:\\nonexistent\\path"
    } else {
        "/nonexistent/path"
    };
    let output = cmd()
        .arg(nonexistent_path)
        .assert()
        .success() // Tree continues even with directory read errors
        .get_output()
        .stderr
        .clone();

    // Check for platform-specific error messages
    let has_unix_error = output_contains(&output, "No such file or directory");
    let has_windows_error = output_contains(&output, "cannot find the path")
        || output_contains(&output, "system cannot find")
        || output_contains(
            &output,
            "The filename, directory name, or volume label syntax is incorrect",
        );
    assert!(
        has_unix_error || has_windows_error,
        "Expected error message not found in stderr: {}",
        String::from_utf8_lossy(&output)
    );

    // Test with invalid level
    cmd().args(["-L", "invalid"]).assert().failure();
}
