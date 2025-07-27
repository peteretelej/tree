// Tests for path normalization functionality
// Since normalize_path is private, we test it indirectly through parsing functions
// that use it, to ensure Windows paths and ./ prefixes are handled correctly

use rust_tree::rust_tree::fromfile::{parse_simple_paths, parse_tar_simple_line};

#[test]
fn test_path_normalization_windows_backslashes() {
    // Test through simple path parsing
    let windows_paths = vec![
        "src\\main.rs".to_string(),
        "test\\path\\file.txt".to_string(),
    ];

    let entries = parse_simple_paths(windows_paths);

    // All paths should be normalized to Unix format
    for entry in &entries {
        assert!(
            !entry.path.contains('\\'),
            "Path contains backslash: {}",
            entry.path
        );
    }

    // Check specific normalized paths
    let has_normalized_main = entries.iter().any(|e| e.path == "src/main.rs");
    let has_normalized_test = entries.iter().any(|e| e.path == "test/path/file.txt");

    assert!(has_normalized_main);
    assert!(has_normalized_test);
}

#[test]
fn test_path_normalization_windows_drive_letters() {
    // Test through TAR simple line parsing
    let c_drive_path = "C:/path/file.txt";
    let entry = parse_tar_simple_line(c_drive_path).unwrap();
    assert_eq!(entry.path, "C/path/file.txt");

    let d_drive_path = "D:\\Users\\test";
    let entry = parse_tar_simple_line(d_drive_path).unwrap();
    assert_eq!(entry.path, "D/Users/test");
}

#[test]
fn test_path_normalization_dot_slash_prefix() {
    // Test through ZIP simple line parsing (we need to construct a valid ZIP line)
    let paths = vec!["./src/main.rs".to_string(), "./file.txt".to_string()];

    let entries = parse_simple_paths(paths);

    // Check that ./ prefix is removed
    let has_src_main = entries.iter().any(|e| e.path == "src/main.rs");
    let has_file = entries.iter().any(|e| e.path == "file.txt");

    assert!(has_src_main);
    assert!(has_file);
}

#[test]
fn test_path_normalization_unchanged_paths() {
    // Test paths that should remain unchanged
    let normal_paths = vec![
        "src/main.rs".to_string(),
        "file.txt".to_string(),
        "".to_string(),
    ];

    let entries = parse_simple_paths(normal_paths);

    let has_src_main = entries.iter().any(|e| e.path == "src/main.rs");
    let has_file = entries.iter().any(|e| e.path == "file.txt");

    assert!(has_src_main);
    assert!(has_file);
}

#[test]
fn test_path_normalization_edge_cases() {
    // Test edge case where path starts with drive but no colon
    let edge_case_path = "C_file.txt";
    let entry = parse_tar_simple_line(edge_case_path).unwrap();
    assert_eq!(entry.path, "C_file.txt"); // Should remain unchanged
}
