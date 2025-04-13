use std::fs::{self, create_dir_all};
use std::process::Command;

fn run_cmd(arg: &[&str]) -> String {
    let binary_path = if cfg!(windows) {
        "target\\debug\\tree.exe"
    } else {
        "target/debug/tree"
    };

    let stdout = Command::new(binary_path)
        .args(arg)
        .output()
        .expect("command failed")
        .stdout;
    String::from_utf8(stdout).expect("Bad parsing")
}

// Helper to create a test directory with empty subdirectories
fn create_empty_dir_fixture() -> String {
    let base = format!("tests/dynamic/empty_{}", std::process::id());
    create_dir_all(&base).unwrap();

    // Create empty directory structure
    create_dir_all(format!("{}/empty1/empty1_1", base)).unwrap();
    create_dir_all(format!("{}/empty2", base)).unwrap();

    base
}

fn cleanup_dynamic_fixture(path: &str) {
    let _ = fs::remove_dir_all(path);
}

// Platform-aware path assertion helper
fn assert_contains_path(haystack: &str, needle: &str) {
    let unix_style = needle.replace('\\', "/");
    let windows_style = needle.replace('/', "\\");
    assert!(
        haystack.contains(&unix_style) || haystack.contains(&windows_style),
        "Output should contain path '{}' (or Windows equivalent '{}')",
        unix_style,
        windows_style
    );
}

#[test]
fn test_basic_tree_structure() {
    let output = run_cmd(&["tests/fixtures/basic"]);
    assert!(output.contains("basic"));
    assert!(output.contains("dir1"));
    assert!(output.contains("dir2"));
    assert!(output.contains("file1.txt"));
}

#[test]
fn test_empty_directories() {
    let base = create_empty_dir_fixture();

    // Test empty directories are shown
    let output = run_cmd(&[&base]);
    assert!(output.contains("empty1"));
    assert!(output.contains("empty1_1"));
    assert!(output.contains("empty2"));
    assert!(output.contains("0 files")); // Should show zero files

    // Test depth limiting with empty dirs
    let limited = run_cmd(&["-L", "1", &base]);
    assert!(limited.contains("empty1"));
    assert!(!limited.contains("empty1_1"));

    cleanup_dynamic_fixture(&base);
}

#[test]
fn test_depth_control() {
    let output = run_cmd(&["-L", "1", "tests/fixtures/basic"]);
    assert!(output.contains("dir1"));
    assert!(!output.contains("dir1_1")); // Shouldn't show deeper level
    assert!(output.contains("dir2"));
}

#[test]
fn test_hidden_files() {
    let normal = run_cmd(&["tests/fixtures/hidden"]);
    let with_hidden = run_cmd(&["-a", "tests/fixtures/hidden"]);

    assert!(
        !normal.contains(".hidden.txt"),
        "Hidden files should not be visible by default"
    );
    assert!(
        with_hidden.contains(".hidden.txt"),
        "Hidden files should be visible with -a flag"
    );
}

#[test]
fn test_formatting_options() {
    // Test full path
    let full_path = run_cmd(&["-f", "tests/fixtures/basic"]);
    assert_contains_path(&full_path, "basic/dir1/file2.txt");

    // Test no indentation
    let no_indent = run_cmd(&["-i", "tests/fixtures/basic"]);
    // Check for both possible indent markers (Unix and Windows)
    assert!(
        !no_indent.contains("├──")
            && !no_indent.contains("└──")
            && !no_indent.contains("+---")
            && !no_indent.contains("\\---")
    );
}

#[test]
fn test_pattern_and_dir_only() {
    // Test pattern matching
    let pattern = run_cmd(&["-P", "*.txt", "tests/fixtures/basic"]);
    assert!(pattern.contains("file1.txt"));
    assert!(pattern.contains("file2.txt"));

    // Test pattern matching with hidden files
    let pattern_with_hidden = run_cmd(&["-P", "*.txt", "-a", "tests/fixtures/hidden"]);
    assert!(pattern_with_hidden.contains(".hidden.txt"));

    // Test directory only
    let dir_only = run_cmd(&["-d", "tests/fixtures/basic"]);
    assert!(dir_only.contains("dir1"));
    assert!(!dir_only.contains("file1.txt"));
}

#[test]
fn test_size_options() {
    let base = format!("tests/dynamic/size_{}", std::process::id());
    create_dir_all(&base).unwrap();
    fs::write(format!("{}/sized_file.txt", base), "x".repeat(1024)).unwrap();

    let size = run_cmd(&["-s", &base]);
    assert!(size.contains("1024"));

    let human = run_cmd(&["-h", &base]);
    assert!(human.contains("1.0K") || human.contains("1.0 KB"));

    cleanup_dynamic_fixture(&base);
}

#[test]
fn test_color_options() {
    // Test color output
    let color = run_cmd(&["-C", "tests/fixtures/basic"]);
    assert!(color.contains("\x1b["), "Should contain ANSI color codes");

    // Test no color
    let no_color = run_cmd(&["-n", "tests/fixtures/basic"]);
    assert!(
        !no_color.contains("\x1b["),
        "Should not contain ANSI color codes"
    );
}

#[test]
fn test_ascii_mode() {
    // Test ASCII mode (for Windows compatibility)
    let ascii = run_cmd(&["-A", "tests/fixtures/basic"]);
    assert!(ascii.contains("|") && ascii.contains("+---"));

    // Test default mode
    let default = run_cmd(&["tests/fixtures/basic"]);
    // Should contain either ASCII or Unicode box drawing chars
    assert!(
        (default.contains("│") && default.contains("├──"))
            || (default.contains("|") && default.contains("+---"))
    );
}
