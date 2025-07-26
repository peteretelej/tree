// Unit tests for display.rs
// Tests colorization and formatting functions

use rust_tree::rust_tree::display::{colorize, format_permissions_unix};
use std::fs;

// Helper function to get a test DirEntry
fn with_test_entry<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&fs::DirEntry) -> R,
{
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            return Some(f(&entry));
        }
    }
    None
}

#[test]
fn test_colorize_function() {
    // Test that colorize function works without panicking
    if let Some(results) = with_test_entry(|entry| {
        let result1 = colorize(entry, "test.txt");
        let result2 = colorize(entry, "main.rs");
        let result3 = colorize(entry, "directory");

        // Function should be consistent
        let consistent1 = colorize(entry, "same");
        let consistent2 = colorize(entry, "same");

        (result1, result2, result3, consistent1, consistent2)
    }) {
        let (result1, result2, result3, consistent1, consistent2) = results;

        // Should return non-empty strings
        assert!(!result1.is_empty());
        assert!(!result2.is_empty());
        assert!(!result3.is_empty());

        // Function should be consistent
        assert_eq!(consistent1, consistent2);
    }
}

#[test]
fn test_colorize_different_extensions() {
    // Test colorize function with different text inputs
    with_test_entry(|entry| {
        let extensions = [
            "test.rs",
            "script.py",
            "data.json",
            "config.toml",
            "image.png",
            "archive.zip",
            "archive.tar",
            "archive.gz",
            "executable",
        ];

        for filename in extensions {
            let result = colorize(entry, filename);
            assert!(!result.is_empty());
            // Function should handle various extensions without panicking
        }
    });
}

#[test]
fn test_colorize_consistency() {
    // Test that function returns consistent results
    if let Some((result1, result2, result3)) = with_test_entry(|entry| {
        let text = "test.txt";
        let result1 = colorize(entry, text);
        let result2 = colorize(entry, text);
        let result3 = colorize(entry, text);
        (result1, result2, result3)
    }) {
        assert_eq!(result1, result2);
        assert_eq!(result2, result3);
    }
}

#[cfg(unix)]
#[test]
fn test_format_permissions_unix() {
    // Test various permission combinations with correct signature
    let formatted_755 = format_permissions_unix(0o755, false);
    let formatted_644 = format_permissions_unix(0o644, false);
    let formatted_777 = format_permissions_unix(0o777, false);
    let formatted_000 = format_permissions_unix(0o000, false);

    // Check that permissions are formatted correctly (with file type indicator)
    assert_eq!(formatted_755, "[-rwxr-xr-x]");
    assert_eq!(formatted_644, "[-rw-r--r--]");
    assert_eq!(formatted_777, "[-rwxrwxrwx]");
    assert_eq!(formatted_000, "[----------]");
}

#[cfg(unix)]
#[test]
fn test_format_permissions_directory_vs_file() {
    // Test directory vs file indicator
    let file_perms = format_permissions_unix(0o644, false);
    let dir_perms = format_permissions_unix(0o755, true);

    assert_eq!(file_perms, "[-rw-r--r--]");
    assert_eq!(dir_perms, "[drwxr-xr-x]");
}

#[cfg(unix)]
#[test]
fn test_format_permissions_special_bits() {
    // Test special permission bits
    let formatted_sticky = format_permissions_unix(0o1755, true); // Sticky bit
    let formatted_setgid = format_permissions_unix(0o2755, true); // Setgid
    let formatted_setuid = format_permissions_unix(0o4755, false); // Setuid

    // Should handle special bits correctly
    assert!(!formatted_sticky.is_empty());
    assert!(!formatted_setgid.is_empty());
    assert!(!formatted_setuid.is_empty());

    // Should be properly formatted with brackets
    assert!(formatted_sticky.starts_with('[') && formatted_sticky.ends_with(']'));
    assert!(formatted_setgid.starts_with('[') && formatted_setgid.ends_with(']'));
    assert!(formatted_setuid.starts_with('[') && formatted_setuid.ends_with(']'));
}

#[cfg(not(unix))]
#[test]
fn test_format_permissions_non_unix() {
    // Test non-Unix platforms return empty string
    let result = format_permissions_unix(0o755, false);
    assert_eq!(result, "");
}

#[test]
fn test_colorize_edge_cases() {
    // Test edge cases with actual DirEntry
    with_test_entry(|entry| {
        // Test various text inputs
        let edge_cases = [
            "",
            "a",
            "a_very_long_filename_that_might_cause_issues.extension",
            "file-with-special_chars.123",
            "file.with.multiple.dots",
            "UPPERCASE.EXT",
        ];

        for text in edge_cases {
            let result = colorize(entry, text);
            // Even empty text should return something (at minimum the text itself)
            assert!(result.len() >= text.len());
        }
    });
}
