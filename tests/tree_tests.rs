use std::fs::{self, create_dir_all};
use std::process::Command;
use std::path::PathBuf;

fn run_cmd(arg: &[&str]) -> String {
    let binary_path = if cfg!(windows) {
        "target\\debug\\tree.exe"
    } else {
        "target/debug/tree"
    };

    let output_result = Command::new(binary_path)
        .args(arg)
        .output()
        .expect("command failed");

    // Print stderr for debugging purposes, especially if the command failed
    if !output_result.stderr.is_empty() {
        eprintln!("--- Captured STDERR for {:?} ---", arg);
        eprintln!("{}", String::from_utf8_lossy(&output_result.stderr));
        eprintln!("--------------------------------------");
    }

    // Panic if the command execution itself failed
    if !output_result.status.success() {
        panic!(
            "Command {:?} failed with status: {}\nStderr:\n{}",
            arg,
            output_result.status,
            String::from_utf8_lossy(&output_result.stderr)
        );
    }

    String::from_utf8(output_result.stdout).expect("Bad parsing")
}

// Helper to create a test directory with a defined structure
fn create_fixture(base_name: &str, files: &[(&str, Option<&str>)]) -> String {
    let base = format!("tests/dynamic/{}_{}", base_name, std::process::id());
    create_dir_all(&base).unwrap();
    for (path, content) in files {
        let full_path = PathBuf::from(&base).join(path);
        if let Some(parent) = full_path.parent() {
            create_dir_all(parent).unwrap();
        }
        if let Some(text) = content {
            fs::write(full_path, text).unwrap();
        } else {
            // Assume directory if content is None
            create_dir_all(full_path).unwrap();
        }
    }
    base
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
    let contains = haystack.contains(&unix_style) || haystack.contains(&windows_style);
    if !contains {
        eprintln!("--- ASSERTION FAILED ---");
        eprintln!("Needle (Unix):    {}", unix_style);
        eprintln!("Needle (Windows): {}", windows_style);
        eprintln!("Haystack:\n{}", haystack);
        eprintln!("-----------------------");
    }
    assert!(
        contains,
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
fn test_exact_output_simple() {
    let base = create_fixture("exact_simple", &[
        ("dir_a/file_a1.txt", Some("a1")),
        ("dir_b/", None), // Empty dir
        ("file_root.txt", Some("root")),
    ]);

    let output = run_cmd(&[&base]);
    let base_name = PathBuf::from(&base).file_name().unwrap().to_string_lossy().to_string();

    // Construct expected output carefully, sensitive to platform line endings potentially
    let expected_lines = [
        base_name.as_str(),
        "├── dir_a",
        "│   └── file_a1.txt",
        "├── dir_b",
        "└── file_root.txt",
        "", // Empty line before summary
        "2 directories, 2 files"
    ];
    let expected_output = expected_lines.join("\n");

    // Normalize line endings in actual output for comparison
    let normalized_output = output.trim_end().replace("\r\n", "\n");

    assert_eq!(normalized_output, expected_output, "Output mismatch for exact structure test");

    cleanup_dynamic_fixture(&base);
}

#[test]
fn test_hidden_directory() {
    let base = create_fixture("hidden_dir", &[
        (".hiddendir/file_in_hidden.txt", Some("hidden")),
        ("visible_file.txt", Some("visible")),
    ]);

    let default_output = run_cmd(&[&base]);
    assert!(!default_output.contains(".hiddendir"));
    assert!(!default_output.contains("file_in_hidden.txt"));
    assert!(default_output.contains("visible_file.txt"));

    let hidden_output = run_cmd(&["-a", &base]);
    assert!(hidden_output.contains(".hiddendir"));
    assert!(hidden_output.contains("file_in_hidden.txt"));
    assert!(hidden_output.contains("visible_file.txt"));

    cleanup_dynamic_fixture(&base);
}

#[test]
fn test_combined_options() {
    let base = create_fixture("combined", &[
        ("dir1/subdir1/file1.txt", Some("abc")),
        ("dir1/.hidden_file", Some("hidden")),
        ("dir2/file2.log", Some("log data")),
    ]);

    // Restore original flags
    let output = run_cmd(&["-L", "2", "-s", "-f", "-a", &base]);

    // Create the base path string for assertions
    let base_path_buf = PathBuf::from(&base);
    let base_path_str = base_path_buf.to_string_lossy();

    // Check level limit (-L 2 means depths 0 and 1 are processed)
    // Since -f is used, check for full paths. Need to account for prefixes.
    // Check that lines ENDING with the expected paths exist.
    let lines: Vec<&str> = output.lines().collect();

    let dir1_path = format!("{}{}dir1", base_path_str, std::path::MAIN_SEPARATOR);
    assert!(lines.iter().any(|line| line.ends_with(&dir1_path)), "Output should contain line ending with: {}", dir1_path);

    let hidden_file_path = format!("{}{}dir1{}.hidden_file", base_path_str, std::path::MAIN_SEPARATOR, std::path::MAIN_SEPARATOR);
    assert!(lines.iter().any(|line| line.contains(&hidden_file_path) && line.contains("6B]")), "Output should contain line with: {} and size", hidden_file_path);

    let subdir1_path = format!("{}{}dir1{}subdir1", base_path_str, std::path::MAIN_SEPARATOR, std::path::MAIN_SEPARATOR);
    assert!(lines.iter().any(|line| line.ends_with(&subdir1_path)), "Output should contain line ending with: {}", subdir1_path);

    let file2_log_path = format!("{}{}dir2{}file2.log", base_path_str, std::path::MAIN_SEPARATOR, std::path::MAIN_SEPARATOR);
    assert!(lines.iter().any(|line| line.contains(&file2_log_path) && line.contains("8B]")), "Output should contain line with: {} and size", file2_log_path);

    // Check that deeper paths are NOT present
    assert!(!output.contains("file1.txt"), "file1.txt (depth 2) should not be present with -L 2");

    cleanup_dynamic_fixture(&base);
}

#[test]
fn test_size_options() {
    let base = format!("tests/dynamic/size_{}", std::process::id());
    create_dir_all(&base).unwrap();
    fs::write(format!("{}/sized_file.txt", base), "x".repeat(1024)).unwrap();
    fs::write(format!("{}/bytes_file.txt", base), "xyz").unwrap();
    fs::write(format!("{}/megabyte_file.bin", base), vec![0u8; 1024 * 1024]).unwrap(); // 1 MB file
    fs::write(format!("{}/zero_byte.txt", base), "").unwrap(); // 0 byte file

    let size = run_cmd(&["-s", &base]);
    // Assertions updated for bracketed format: " [ padding B]"
    assert!(size.contains("[    3B]")); // bytes_file.txt
    assert!(size.contains("[ 1024B]")); // sized_file.txt
    assert!(size.contains("[1048576B]")); // megabyte_file.bin (exact bytes)
    assert!(size.contains("[    0B]")); // zero_byte.txt

    let human = run_cmd(&["-H", &base]);
    // Assertions updated for bracketed format: " [size UNIT]"
    assert!(human.contains("[3.0 B]") || human.contains("[3 B]"));
    assert!(human.contains("[1.0 KB]") || human.contains("[1.0K]"));
    assert!(human.contains("[1.0 MB]") || human.contains("[1.0M]"));
    assert!(human.contains("[0.0 B]") || human.contains("[0 B]"));

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

#[test]
fn test_exclude_pattern() {
    // Test basic exclude
    let output = run_cmd(&["-I", "*.txt", "tests/fixtures/basic"]);
    assert!(!output.contains("file1.txt"), "Should not show .txt files");
    assert!(!output.contains("file2.txt"), "Should not show .txt files");
    assert!(output.contains("dir1"), "Should still show directories");

    // Test exclude with hidden files
    let with_hidden = run_cmd(&["-I", "*.txt", "-a", "tests/fixtures/hidden"]);
    assert!(!with_hidden.contains(".hidden.txt"), "Should not show hidden .txt files when excluded");

    // Test exclude pattern with directories
    let exclude_dir = run_cmd(&["-I", "dir1", "tests/fixtures/basic"]);
    assert!(!exclude_dir.contains("dir1"), "Should not show excluded directory");
    assert!(exclude_dir.contains("dir2"), "Should show non-excluded directory");

    // Test multiple patterns (glob crate doesn't support `|` directly in one pattern)
    // Need to handle this in the argument parsing or logic if required,
    // but current implementation likely treats it as a literal filename part.
    // For now, test exclusion of a file with '|' if the pattern syntax supported it.
    // Let's assume the glob library handles basic wildcards well.
    let exclude_specific = run_cmd(&["-I", "file1.txt", "tests/fixtures/basic"]);
    assert!(!exclude_specific.contains("file1.txt"));
    assert!(exclude_specific.contains("file2.txt"));
    assert!(exclude_specific.contains("dir1"));
}
