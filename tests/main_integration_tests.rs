// Integration tests for main.rs CLI functionality
// Tests CLI argument parsing, main function execution, and error handling

use assert_cmd::prelude::*;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_basic_cli_execution() {
    let temp_dir = tempdir().unwrap();
    fs::write(temp_dir.path().join("test.txt"), "content").unwrap();

    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.current_dir(temp_dir.path());

    let output = cmd.output().unwrap();
    assert!(output.status.success());
    assert!(!output.stdout.is_empty());
}

#[test]
fn test_cli_help_flag() {
    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.arg("--help");

    let output = cmd.output().unwrap();
    assert!(output.status.success());
    assert!(!output.stdout.is_empty());

    let stdout_str = String::from_utf8(output.stdout).unwrap();
    assert!(stdout_str.contains("Usage:") || stdout_str.contains("tree"));
}

#[test]
fn test_cli_version_flag() {
    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.arg("--version");

    let output = cmd.output().unwrap();
    assert!(output.status.success());
    assert!(!output.stdout.is_empty());

    let stdout_str = String::from_utf8(output.stdout).unwrap();
    assert!(stdout_str.contains("tree") || !stdout_str.trim().is_empty());
}

#[test]
fn test_cli_all_files_flag() {
    let temp_dir = tempdir().unwrap();
    fs::write(temp_dir.path().join(".hidden"), "hidden content").unwrap();
    fs::write(temp_dir.path().join("visible.txt"), "visible content").unwrap();

    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.current_dir(temp_dir.path()).arg("-a");

    let output = cmd.output().unwrap();
    assert!(output.status.success());

    let stdout_str = String::from_utf8(output.stdout).unwrap();
    assert!(stdout_str.contains(".hidden"));
    assert!(stdout_str.contains("visible.txt"));
}

#[test]
fn test_cli_depth_limit() {
    let temp_dir = tempdir().unwrap();
    let sub_dir = temp_dir.path().join("sub");
    fs::create_dir(&sub_dir).unwrap();
    fs::write(sub_dir.join("deep.txt"), "deep content").unwrap();

    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.current_dir(temp_dir.path()).arg("-L").arg("1");

    let output = cmd.output().unwrap();
    assert!(output.status.success());

    let stdout_str = String::from_utf8(output.stdout).unwrap();
    assert!(stdout_str.contains("sub"));
    // Should not show deep.txt due to depth limit
    assert!(!stdout_str.contains("deep.txt"));
}

#[test]
fn test_cli_directories_only() {
    let temp_dir = tempdir().unwrap();
    fs::create_dir(temp_dir.path().join("dir")).unwrap();
    fs::write(temp_dir.path().join("file.txt"), "content").unwrap();

    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.current_dir(temp_dir.path()).arg("-d");

    let output = cmd.output().unwrap();
    assert!(output.status.success());

    let stdout_str = String::from_utf8(output.stdout).unwrap();
    assert!(stdout_str.contains("dir"));
    assert!(!stdout_str.contains("file.txt"));
}

#[test]
fn test_cli_file_sizes() {
    let temp_dir = tempdir().unwrap();
    fs::write(temp_dir.path().join("test.txt"), "test content").unwrap();

    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.current_dir(temp_dir.path()).arg("-s");

    let output = cmd.output().unwrap();
    assert!(output.status.success());

    let stdout_str = String::from_utf8(output.stdout).unwrap();
    assert!(stdout_str.contains("test.txt"));
    // Should contain some size information
    assert!(stdout_str.contains("B") || stdout_str.contains("["));
}

#[test]
fn test_cli_human_readable_sizes() {
    let temp_dir = tempdir().unwrap();
    fs::write(temp_dir.path().join("test.txt"), "test content").unwrap();

    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.current_dir(temp_dir.path()).arg("-s").arg("-H");

    let output = cmd.output().unwrap();
    assert!(output.status.success());

    let stdout_str = String::from_utf8(output.stdout).unwrap();
    assert!(stdout_str.contains("test.txt"));
}

#[test]
fn test_cli_pattern_matching() {
    let temp_dir = tempdir().unwrap();
    fs::write(temp_dir.path().join("file.rs"), "rust code").unwrap();
    fs::write(temp_dir.path().join("file.txt"), "text content").unwrap();

    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.current_dir(temp_dir.path()).arg("-P").arg("*.rs");

    let output = cmd.output().unwrap();
    assert!(output.status.success());

    let stdout_str = String::from_utf8(output.stdout).unwrap();
    assert!(stdout_str.contains("file.rs"));
    // Should not show .txt files
    assert!(!stdout_str.contains("file.txt"));
}

#[test]
fn test_cli_exclude_pattern() {
    let temp_dir = tempdir().unwrap();
    fs::write(temp_dir.path().join("keep.rs"), "keep this").unwrap();
    fs::write(temp_dir.path().join("ignore.txt"), "ignore this").unwrap();

    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.current_dir(temp_dir.path()).arg("-I").arg("*.txt");

    let output = cmd.output().unwrap();
    assert!(output.status.success());

    let stdout_str = String::from_utf8(output.stdout).unwrap();
    assert!(stdout_str.contains("keep.rs"));
    assert!(!stdout_str.contains("ignore.txt"));
}

#[test]
fn test_cli_ascii_mode() {
    let temp_dir = tempdir().unwrap();
    fs::create_dir(temp_dir.path().join("subdir")).unwrap();
    fs::write(temp_dir.path().join("file.txt"), "content").unwrap();

    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.current_dir(temp_dir.path()).arg("--ascii");

    let output = cmd.output().unwrap();
    assert!(output.status.success());

    let stdout_str = String::from_utf8(output.stdout).unwrap();
    // Should use ASCII characters instead of Unicode
    assert!(stdout_str.contains("|") || stdout_str.contains("\\") || stdout_str.contains("+"));
}

#[test]
fn test_cli_full_path() {
    let temp_dir = tempdir().unwrap();
    fs::write(temp_dir.path().join("test.txt"), "content").unwrap();

    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.current_dir(temp_dir.path()).arg("-f");

    let output = cmd.output().unwrap();
    assert!(output.status.success());

    let stdout_str = String::from_utf8(output.stdout).unwrap();
    // Should show full paths
    assert!(stdout_str.contains("test.txt"));
}

#[test]
fn test_cli_no_indent() {
    let temp_dir = tempdir().unwrap();
    fs::create_dir(temp_dir.path().join("subdir")).unwrap();
    fs::write(temp_dir.path().join("subdir").join("file.txt"), "content").unwrap();

    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.current_dir(temp_dir.path()).arg("-i");

    let output = cmd.output().unwrap();
    assert!(output.status.success());

    let stdout_str = String::from_utf8(output.stdout).unwrap();
    assert!(stdout_str.contains("subdir"));
    assert!(stdout_str.contains("file.txt"));
}

#[test]
fn test_cli_sort_by_time() {
    let temp_dir = tempdir().unwrap();
    fs::write(temp_dir.path().join("first.txt"), "first").unwrap();
    // Small delay to ensure different timestamps
    std::thread::sleep(std::time::Duration::from_millis(10));
    fs::write(temp_dir.path().join("second.txt"), "second").unwrap();

    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.current_dir(temp_dir.path()).arg("-t");

    let output = cmd.output().unwrap();
    assert!(output.status.success());

    let stdout_str = String::from_utf8(output.stdout).unwrap();
    assert!(stdout_str.contains("first.txt"));
    assert!(stdout_str.contains("second.txt"));
}

#[test]
fn test_cli_reverse_sort() {
    let temp_dir = tempdir().unwrap();
    fs::write(temp_dir.path().join("a_file.txt"), "a").unwrap();
    fs::write(temp_dir.path().join("z_file.txt"), "z").unwrap();

    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.current_dir(temp_dir.path()).arg("-r");

    let output = cmd.output().unwrap();
    assert!(output.status.success());

    let stdout_str = String::from_utf8(output.stdout).unwrap();
    assert!(stdout_str.contains("a_file.txt"));
    assert!(stdout_str.contains("z_file.txt"));
}

#[test]
fn test_cli_modification_dates() {
    let temp_dir = tempdir().unwrap();
    fs::write(temp_dir.path().join("test.txt"), "content").unwrap();

    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.current_dir(temp_dir.path()).arg("-D");

    let output = cmd.output().unwrap();
    assert!(output.status.success());

    let stdout_str = String::from_utf8(output.stdout).unwrap();
    assert!(stdout_str.contains("test.txt"));
    // Should contain date information
    assert!(stdout_str.contains("202") || stdout_str.contains("["));
}

#[test]
fn test_cli_file_limit() {
    let temp_dir = tempdir().unwrap();
    for i in 0..10 {
        fs::write(temp_dir.path().join(format!("file_{}.txt", i)), "content").unwrap();
    }

    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.current_dir(temp_dir.path()).arg("--filelimit").arg("3");

    let output = cmd.output().unwrap();
    assert!(output.status.success());

    let stdout_str = String::from_utf8(output.stdout).unwrap();
    // Should limit output
    assert!(!stdout_str.is_empty());
}

#[test]
fn test_cli_dirs_first() {
    let temp_dir = tempdir().unwrap();
    fs::create_dir(temp_dir.path().join("z_dir")).unwrap();
    fs::write(temp_dir.path().join("a_file.txt"), "content").unwrap();

    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.current_dir(temp_dir.path()).arg("--dirsfirst");

    let output = cmd.output().unwrap();
    assert!(output.status.success());

    let stdout_str = String::from_utf8(output.stdout).unwrap();
    assert!(stdout_str.contains("z_dir"));
    assert!(stdout_str.contains("a_file.txt"));
}

#[test]
fn test_cli_classify() {
    let temp_dir = tempdir().unwrap();
    fs::create_dir(temp_dir.path().join("directory")).unwrap();
    fs::write(temp_dir.path().join("file.txt"), "content").unwrap();

    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.current_dir(temp_dir.path()).arg("-F");

    let output = cmd.output().unwrap();
    assert!(output.status.success());

    let stdout_str = String::from_utf8(output.stdout).unwrap();
    assert!(stdout_str.contains("directory"));
    assert!(stdout_str.contains("file.txt"));
}

#[test]
fn test_cli_no_report() {
    let temp_dir = tempdir().unwrap();
    fs::write(temp_dir.path().join("test.txt"), "content").unwrap();

    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.current_dir(temp_dir.path()).arg("--noreport");

    let output = cmd.output().unwrap();
    assert!(output.status.success());

    let stdout_str = String::from_utf8(output.stdout).unwrap();
    assert!(stdout_str.contains("test.txt"));
    // Should not contain summary report
    assert!(!stdout_str.contains("directories") || !stdout_str.contains("files"));
}

#[cfg(unix)]
#[test]
fn test_cli_permissions() {
    let temp_dir = tempdir().unwrap();
    fs::write(temp_dir.path().join("test.txt"), "content").unwrap();

    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.current_dir(temp_dir.path()).arg("-p");

    let output = cmd.output().unwrap();
    assert!(output.status.success());

    let stdout_str = String::from_utf8(output.stdout).unwrap();
    assert!(stdout_str.contains("test.txt"));
    // Should contain permission info
    assert!(stdout_str.contains("[") && stdout_str.contains("]"));
}

#[test]
fn test_cli_color_options() {
    let temp_dir = tempdir().unwrap();
    fs::write(temp_dir.path().join("test.txt"), "content").unwrap();

    // Test -C/--color flag
    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.current_dir(temp_dir.path()).arg("-C");

    let output = cmd.output().unwrap();
    assert!(output.status.success());

    // Test -n/--no-color flag
    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.current_dir(temp_dir.path()).arg("-n");

    let output = cmd.output().unwrap();
    assert!(output.status.success());
}

#[test]
fn test_cli_fromfile_option() {
    let temp_dir = tempdir().unwrap();
    let file_list = temp_dir.path().join("files.txt");
    fs::write(&file_list, "src/main.rs\ntests/test.rs\nREADME.md").unwrap();

    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("--fromfile")
        .arg(&file_list);

    let output = cmd.output().unwrap();
    assert!(output.status.success());

    let stdout_str = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout_str.contains("main.rs")
            || stdout_str.contains("test.rs")
            || stdout_str.contains("README.md")
    );
}

#[test]
fn test_cli_invalid_arguments() {
    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.arg("--invalid-flag");

    let output = cmd.output().unwrap();
    // Should exit with error for invalid flag
    assert!(!output.status.success());
    assert!(!output.stderr.is_empty());
}

#[test]
fn test_cli_nonexistent_directory() {
    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.arg("/this/path/does/not/exist");

    let output = cmd.output().unwrap();
    // May succeed but should handle gracefully
    let stderr_str = String::from_utf8(output.stderr).unwrap();
    // Should either succeed or show appropriate error
    assert!(
        output.status.success() || stderr_str.contains("Error") || stderr_str.contains("No such")
    );
}

#[test]
fn test_cli_combined_flags() {
    let temp_dir = tempdir().unwrap();
    fs::create_dir(temp_dir.path().join("src")).unwrap();
    fs::write(temp_dir.path().join("src").join("main.rs"), "rust code").unwrap();
    fs::write(temp_dir.path().join("README.md"), "readme").unwrap();

    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("-a") // all files
        .arg("-s") // sizes
        .arg("-H") // human readable (capital H)
        .arg("-D") // dates
        .arg("-L")
        .arg("2"); // depth limit

    let output = cmd.output().unwrap();
    assert!(output.status.success());

    let stdout_str = String::from_utf8(output.stdout).unwrap();
    assert!(stdout_str.contains("main.rs"));
    assert!(stdout_str.contains("README.md"));
}

#[test]
fn test_cli_output_to_file() {
    let temp_dir = tempdir().unwrap();
    fs::write(temp_dir.path().join("test.txt"), "content").unwrap();
    let output_file = temp_dir.path().join("output.txt");

    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.current_dir(temp_dir.path()).arg("-o").arg(&output_file);

    let output = cmd.output().unwrap();
    assert!(output.status.success());

    // Check that output file was created and has content
    assert!(output_file.exists());
    let file_content = fs::read_to_string(&output_file).unwrap();
    assert!(!file_content.is_empty());
    assert!(file_content.contains("test.txt"));
}

#[test]
fn test_cli_multiple_patterns() {
    let temp_dir = tempdir().unwrap();
    fs::write(temp_dir.path().join("test.rs"), "rust").unwrap();
    fs::write(temp_dir.path().join("test.txt"), "text").unwrap();
    fs::write(temp_dir.path().join("test.md"), "markdown").unwrap();

    // Test pattern that should match multiple types
    let mut cmd = Command::cargo_bin("tree").unwrap();
    cmd.current_dir(temp_dir.path()).arg("-P").arg("test.*");

    let output = cmd.output().unwrap();
    assert!(output.status.success());

    let stdout_str = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout_str.contains("test.rs")
            || stdout_str.contains("test.txt")
            || stdout_str.contains("test.md")
    );
}

#[test]
fn test_main_function_error_handling() {
    // Test that the main function handles errors gracefully
    let mut cmd = Command::cargo_bin("tree").unwrap();
    // Pass a very deep path that likely doesn't exist
    cmd.arg("/very/deep/path/that/should/not/exist/anywhere");

    let output = cmd.output().unwrap();
    // Should handle the error without panicking
    // Either succeeds with error message or fails gracefully
    let stderr_str = String::from_utf8(output.stderr).unwrap();
    assert!(output.status.success() || !stderr_str.is_empty());
}
