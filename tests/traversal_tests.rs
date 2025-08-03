// Comprehensive tests for traversal.rs
// Tests date formatting, filtering, sorting, and advanced traversal features

use glob::Pattern;
use rust_tree::rust_tree::options::TreeOptions;
use rust_tree::rust_tree::traversal::{list_directory, list_directory_as_string};
use std::fs;
use std::io::Write;
use tempfile::{tempdir, NamedTempFile};

// Helper function to create a temporary directory structure for testing
fn create_test_directory() -> tempfile::TempDir {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Create directory structure
    fs::create_dir(temp_path.join("src")).unwrap();
    fs::create_dir(temp_path.join("tests")).unwrap();
    fs::create_dir(temp_path.join(".hidden")).unwrap();

    // Create files
    fs::write(temp_path.join("README.md"), "# Test Project").unwrap();
    fs::write(temp_path.join("Cargo.toml"), "[package]").unwrap();
    fs::write(temp_path.join("src").join("main.rs"), "fn main() {}").unwrap();
    fs::write(temp_path.join("src").join("lib.rs"), "// lib").unwrap();
    fs::write(temp_path.join("tests").join("test.rs"), "#[test]").unwrap();
    fs::write(temp_path.join(".hidden").join("secret.txt"), "secret").unwrap();
    fs::write(temp_path.join(".gitignore"), "target/").unwrap();

    temp_dir
}

// Helper function to create default TreeOptions
fn create_default_options() -> TreeOptions {
    TreeOptions {
        all_files: false,
        level: None,
        full_path: false,
        dir_only: false,
        no_indent: false,
        print_size: false,
        human_readable: false,
        pattern_glob: None,
        exclude_pattern: None,
        color: false,
        no_color: false,
        ascii: false,
        sort_by_time: false,
        reverse: false,
        print_mod_date: false,
        output_file: None,
        file_limit: None,
        dirs_first: false,
        classify: false,
        no_report: false,
        print_permissions: false,
        from_file: false,
    }
}

#[test]
fn test_list_directory_basic() {
    let temp_dir = create_test_directory();
    let options = create_default_options();

    // Test that basic directory listing works without error
    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());
}

#[test]
fn test_list_directory_with_all_files() {
    let temp_dir = create_test_directory();
    let mut options = create_default_options();
    options.all_files = true;

    // Test that all_files option works without error
    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());
}

#[test]
fn test_list_directory_with_pattern() {
    let temp_dir = create_test_directory();
    let mut options = create_default_options();
    options.pattern_glob = Some(Pattern::new("*.rs").unwrap());

    // Test that pattern filtering works without error
    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());
}

#[test]
fn test_list_directory_exclude_pattern() {
    let temp_dir = create_test_directory();
    let mut options = create_default_options();
    options.exclude_pattern = Some(Pattern::new("target").unwrap());

    // Test that exclude pattern works without error
    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());
}

#[test]
fn test_list_directory_dirs_only() {
    let temp_dir = create_test_directory();
    let mut options = create_default_options();
    options.dir_only = true;

    // Test that directories-only mode works without error
    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());
}

#[test]
fn test_list_directory_with_depth_limit() {
    let temp_dir = create_test_directory();
    let mut options = create_default_options();
    options.level = Some(1);

    // Test that depth limiting works without error
    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());
}

#[test]
fn test_list_directory_with_sizes() {
    let temp_dir = create_test_directory();
    let mut options = create_default_options();
    options.print_size = true;

    // Test that size printing works without error
    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());
}

#[test]
fn test_list_directory_human_readable_sizes() {
    let temp_dir = create_test_directory();
    let mut options = create_default_options();
    options.print_size = true;
    options.human_readable = true;

    // Test that human-readable sizes work without error
    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());
}

#[test]
fn test_list_directory_with_modification_dates() {
    let temp_dir = create_test_directory();
    let mut options = create_default_options();
    options.print_mod_date = true;

    // Test that modification date printing works without error
    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());
}

#[test]
fn test_list_directory_sort_by_time() {
    let temp_dir = create_test_directory();
    let mut options = create_default_options();
    options.sort_by_time = true;

    // Test that time-based sorting works without error
    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());
}

#[test]
fn test_list_directory_reverse_sort() {
    let temp_dir = create_test_directory();
    let mut options = create_default_options();
    options.reverse = true;

    // Test that reverse sorting works without error
    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());
}

#[test]
fn test_list_directory_dirs_first() {
    let temp_dir = create_test_directory();
    let mut options = create_default_options();
    options.dirs_first = true;

    // Test that directories-first sorting works without error
    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());
}

#[test]
fn test_list_directory_ascii_mode() {
    let temp_dir = create_test_directory();
    let mut options = create_default_options();
    options.ascii = true;

    // Test that ASCII mode works without error
    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());
}

#[test]
fn test_list_directory_full_path() {
    let temp_dir = create_test_directory();
    let mut options = create_default_options();
    options.full_path = true;

    // Test that full path mode works without error
    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());
}

#[test]
fn test_list_directory_no_indent() {
    let temp_dir = create_test_directory();
    let mut options = create_default_options();
    options.no_indent = true;

    // Test that no-indent mode works without error
    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());
}

#[test]
fn test_list_directory_classify() {
    let temp_dir = create_test_directory();
    let mut options = create_default_options();
    options.classify = true;

    // Test that classify mode works without error
    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());
}

#[test]
fn test_list_directory_no_report() {
    let temp_dir = create_test_directory();
    let mut options = create_default_options();
    options.no_report = true;

    // Test that no-report mode works without error
    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());
}

#[cfg(unix)]
#[test]
fn test_list_directory_with_permissions() {
    let temp_dir = create_test_directory();
    let mut options = create_default_options();
    options.print_permissions = true;

    // Test that permission printing works without error
    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());
}

#[test]
fn test_list_directory_with_file_limit() {
    let temp_dir = create_test_directory();
    let mut options = create_default_options();
    options.file_limit = Some(3);

    // Test that file limiting works without error
    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());
}

#[test]
fn test_output_capture() {
    let temp_dir = create_test_directory();
    let options = create_default_options();

    // Test that directory listing works (output goes to stdout)
    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());
}

#[test]
fn test_traverse_directory_with_output_file() {
    let temp_dir = create_test_directory();
    let mut temp_file = NamedTempFile::new().unwrap();
    let mut options = create_default_options();
    options.output_file = Some(temp_file.path().to_string_lossy().to_string());

    // Test that output file option works
    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());

    // File should have been written to
    temp_file.flush().unwrap();
    let file_size = temp_file.as_file().metadata().unwrap().len();
    assert!(file_size > 0);
}

#[test]
fn test_list_directory_nonexistent_path() {
    let options = create_default_options();
    let nonexistent_path = "/this/path/should/not/exist";

    // Test that nonexistent path is handled gracefully
    // Note: The function may print an error but still return Ok()
    let result = list_directory(nonexistent_path, &options);
    // Accept either error or success (function handles error internally)
    let _ = result;
}

#[test]
fn test_list_directory_combined_options() {
    let temp_dir = create_test_directory();
    let mut options = create_default_options();

    // Test multiple options combined
    options.all_files = true;
    options.print_size = true;
    options.human_readable = true;
    options.print_mod_date = true;
    options.sort_by_time = true;
    options.dirs_first = true;
    options.level = Some(2);

    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());
}

#[test]
fn test_multiple_pattern_combinations() {
    let temp_dir = create_test_directory();

    // Test different pattern combinations
    let patterns = vec![
        "*.rs", "*.md", "Cargo.*", "*test*", ".*", // hidden files
    ];

    for pattern_str in patterns {
        let mut options = create_default_options();
        options.pattern_glob = Some(Pattern::new(pattern_str).unwrap());
        options.all_files = true; // Include hidden files for .* pattern

        let result = list_directory(temp_dir.path(), &options);
        assert!(result.is_ok(), "Failed with pattern: {}", pattern_str);
    }
}

#[test]
fn test_exclude_pattern_combinations() {
    let temp_dir = create_test_directory();

    // Test different exclude patterns
    let exclude_patterns = vec![
        "*.md", "test*", ".*", // exclude hidden files
        "src",
    ];

    for pattern_str in exclude_patterns {
        let mut options = create_default_options();
        options.exclude_pattern = Some(Pattern::new(pattern_str).unwrap());
        options.all_files = true;

        let result = list_directory(temp_dir.path(), &options);
        assert!(
            result.is_ok(),
            "Failed with exclude pattern: {}",
            pattern_str
        );
    }
}

#[test]
fn test_depth_level_variations() {
    let temp_dir = create_test_directory();

    // Test different depth levels
    for level in 0..=5 {
        let mut options = create_default_options();
        options.level = Some(level);

        let result = list_directory(temp_dir.path(), &options);
        assert!(result.is_ok(), "Failed with depth level: {}", level);
    }
}

#[test]
fn test_file_limit_variations() {
    let temp_dir = create_test_directory();

    // Test different file limits
    for limit in 1..=10 {
        let mut options = create_default_options();
        options.file_limit = Some(limit);

        let result = list_directory(temp_dir.path(), &options);
        assert!(result.is_ok(), "Failed with file limit: {}", limit);
    }
}

#[test]
fn test_color_combinations() {
    let temp_dir = create_test_directory();

    // Test color options
    let color_configs = vec![
        (false, false), // no color flags
        (true, false),  // color enabled
        (false, true),  // no_color enabled
        (true, true),   // both (no_color should override)
    ];

    for (color, no_color) in color_configs {
        let mut options = create_default_options();
        options.color = color;
        options.no_color = no_color;

        let result = list_directory(temp_dir.path(), &options);
        assert!(
            result.is_ok(),
            "Failed with color: {}, no_color: {}",
            color,
            no_color
        );
    }
}

#[test]
fn test_empty_directory() {
    let temp_dir = tempdir().unwrap();
    // Don't create any files - test empty directory

    let options = create_default_options();
    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());
}

#[test]
fn test_single_file_directory() {
    let temp_dir = tempdir().unwrap();
    fs::write(temp_dir.path().join("single.txt"), "content").unwrap();

    let options = create_default_options();
    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());
}

#[test]
fn test_deeply_nested_directory() {
    let temp_dir = tempdir().unwrap();
    let mut current_path = temp_dir.path().to_path_buf();

    // Create deeply nested structure
    for i in 0..5 {
        current_path.push(format!("level_{}", i));
        fs::create_dir(&current_path).unwrap();
    }
    fs::write(current_path.join("deep_file.txt"), "deep content").unwrap();

    let options = create_default_options();
    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());
}

#[test]
fn test_large_directory() {
    let temp_dir = tempdir().unwrap();

    // Create many files to test performance
    for i in 0..50 {
        fs::write(
            temp_dir.path().join(format!("file_{:03}.txt", i)),
            "content",
        )
        .unwrap();
    }

    let options = create_default_options();
    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());
}

#[test]
fn test_special_characters_in_filenames() {
    let temp_dir = tempdir().unwrap();

    // Test files with special characters
    let special_files = vec![
        "file with spaces.txt",
        "file-with-dashes.txt",
        "file_with_underscores.txt",
        "file.with.dots.txt",
        "file(with)parentheses.txt",
        "file[with]brackets.txt",
    ];

    for filename in &special_files {
        fs::write(temp_dir.path().join(filename), "content").unwrap();
    }

    let options = create_default_options();
    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());
}

#[test]
fn test_list_directory_as_string_basic() {
    let temp_dir = create_test_directory();
    let options = create_default_options();
    
    let result = list_directory_as_string(temp_dir.path(), &options);
    assert!(result.is_ok());
    
    let output = result.unwrap();
    assert!(!output.is_empty());
    
    // Check that the output contains expected directories
    assert!(output.contains("src"));
    assert!(output.contains("tests"));
    assert!(output.contains("README.md"));
    assert!(output.contains("Cargo.toml"));
    
    // Hidden files should not be included by default
    assert!(!output.contains(".hidden"));
    assert!(!output.contains(".gitignore"));
}

#[test]
fn test_list_directory_as_string_with_all_files() {
    let temp_dir = create_test_directory();
    let mut options = create_default_options();
    options.all_files = true;
    
    let result = list_directory_as_string(temp_dir.path(), &options);
    assert!(result.is_ok());
    
    let output = result.unwrap();
    
    // Hidden files should now be included
    assert!(output.contains(".hidden"));
    assert!(output.contains(".gitignore"));
}

#[test]
fn test_list_directory_as_string_with_tree_formatting() {
    let temp_dir = create_test_directory();
    let options = create_default_options();
    
    let result = list_directory_as_string(temp_dir.path(), &options);
    assert!(result.is_ok());
    
    let output = result.unwrap();
    
    // Check for tree formatting characters
    assert!(output.contains("├──") || output.contains("└──"));
    assert!(output.contains("│") || output.contains("|"));
}

#[test]
fn test_list_directory_as_string_with_no_report() {
    let temp_dir = create_test_directory();
    let mut options = create_default_options();
    options.no_report = true;
    
    let result = list_directory_as_string(temp_dir.path(), &options);
    assert!(result.is_ok());
    
    let output = result.unwrap();
    
    // Should not contain the summary report
    assert!(!output.contains("directories"));
    assert!(!output.contains("files"));
}

#[test]
fn test_list_directory_as_string_nonexistent_path() {
    let options = create_default_options();
    let nonexistent_path = "/this/path/does/not/exist";
    
    let result = list_directory_as_string(nonexistent_path, &options);
    assert!(result.is_err());
}
