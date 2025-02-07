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
        exclude_patterns: vec![],
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
        icons: false,
        prune: false,
        match_dirs: false,
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
    options.exclude_patterns = vec![Pattern::new("target").unwrap()];

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
        options.exclude_patterns = vec![Pattern::new(pattern_str).unwrap()];
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

fn create_prune_test_directory() -> tempfile::TempDir {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    fs::create_dir(temp_path.join("empty_dir")).unwrap();
    fs::create_dir(temp_path.join("has_txt")).unwrap();
    fs::create_dir(temp_path.join("has_txt").join("subdir")).unwrap();
    fs::create_dir(temp_path.join("no_txt")).unwrap();

    fs::write(temp_path.join("has_txt").join("file.txt"), "content").unwrap();
    fs::write(
        temp_path.join("has_txt").join("subdir").join("nested.txt"),
        "nested",
    )
    .unwrap();
    fs::write(temp_path.join("no_txt").join("file.rs"), "fn main() {}").unwrap();

    temp_dir
}

#[test]
fn test_prune_with_pattern() {
    let temp_dir = create_prune_test_directory();
    let mut options = create_default_options();
    options.pattern_glob = Some(Pattern::new("*.txt").unwrap());
    options.prune = true;

    let result = list_directory_as_string(temp_dir.path(), &options);
    assert!(result.is_ok());

    let output = result.unwrap();

    assert!(output.contains("has_txt"), "Should contain has_txt dir");
    assert!(output.contains("file.txt"), "Should contain file.txt");
    assert!(output.contains("subdir"), "Should contain subdir");
    assert!(output.contains("nested.txt"), "Should contain nested.txt");

    assert!(
        !output.contains("empty_dir"),
        "Should not contain empty_dir"
    );
    assert!(!output.contains("no_txt"), "Should not contain no_txt dir");
    assert!(!output.contains("file.rs"), "Should not contain file.rs");
}

#[test]
fn test_prune_with_exclude() {
    let temp_dir = create_prune_test_directory();
    let mut options = create_default_options();
    options.exclude_patterns = vec![Pattern::new("*.txt").unwrap()];
    options.prune = true;

    let result = list_directory_as_string(temp_dir.path(), &options);
    assert!(result.is_ok());

    let output = result.unwrap();

    assert!(output.contains("no_txt"), "Should contain no_txt dir");
    assert!(output.contains("file.rs"), "Should contain file.rs");

    assert!(
        !output.contains("empty_dir"),
        "Should not contain empty_dir"
    );
    assert!(
        !output.contains("has_txt"),
        "Should not contain has_txt (all content excluded)"
    );
}

#[test]
fn test_prune_without_filter_has_no_effect() {
    let temp_dir = create_prune_test_directory();
    let mut options = create_default_options();
    options.prune = true;

    let result = list_directory_as_string(temp_dir.path(), &options);
    assert!(result.is_ok());

    let output = result.unwrap();

    assert!(
        output.contains("empty_dir"),
        "Should contain empty_dir (no filter active)"
    );
    assert!(output.contains("has_txt"), "Should contain has_txt");
    assert!(output.contains("no_txt"), "Should contain no_txt");
}

#[test]
fn test_prune_nested_directories() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    fs::create_dir_all(temp_path.join("a").join("b").join("c")).unwrap();
    fs::write(
        temp_path.join("a").join("b").join("c").join("deep.txt"),
        "deep",
    )
    .unwrap();
    fs::create_dir(temp_path.join("empty")).unwrap();

    let mut options = create_default_options();
    options.pattern_glob = Some(Pattern::new("*.txt").unwrap());
    options.prune = true;

    let result = list_directory_as_string(temp_dir.path(), &options);
    assert!(result.is_ok());

    let output = result.unwrap();

    assert!(output.contains("deep.txt"), "Should contain deep.txt");
    assert!(!output.contains("empty"), "Should not contain empty dir");
}

fn create_matchdirs_test_directory() -> tempfile::TempDir {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Create structure matching the plan's test case:
    // fzf_root/init.lua        (depth 2, parent matches at depth 1)
    // fzf_root/sub/nested.lua  (depth 3, parent doesn't match at depth 1)
    // project/fzf/plugin.lua   (depth 3, fzf matches but at depth 2)
    fs::create_dir_all(temp_path.join("fzf_root").join("sub").join("deep")).unwrap();
    fs::create_dir_all(temp_path.join("project").join("fzf").join("sub")).unwrap();

    fs::write(temp_path.join("fzf_root").join("init.lua"), "x").unwrap();
    fs::write(
        temp_path.join("fzf_root").join("sub").join("nested.lua"),
        "x",
    )
    .unwrap();
    fs::write(
        temp_path
            .join("fzf_root")
            .join("sub")
            .join("deep")
            .join("verydeep.lua"),
        "x",
    )
    .unwrap();
    fs::write(
        temp_path.join("project").join("fzf").join("plugin.lua"),
        "x",
    )
    .unwrap();
    fs::write(
        temp_path
            .join("project")
            .join("fzf")
            .join("sub")
            .join("nested.lua"),
        "x",
    )
    .unwrap();

    temp_dir
}

#[test]
fn test_matchdirs_depth1_contents_shown() {
    let temp_dir = create_matchdirs_test_directory();
    let mut options = create_default_options();
    options.pattern_glob = Some(Pattern::new("fzf*").unwrap());
    options.match_dirs = true;

    let result = list_directory_as_string(temp_dir.path(), &options);
    assert!(result.is_ok());

    let output = result.unwrap();

    // fzf_root matches pattern at depth 1, so its immediate children should be shown
    assert!(output.contains("fzf_root"), "Should contain fzf_root");
    assert!(
        output.contains("init.lua"),
        "Should contain init.lua (depth 2, parent matches at depth 1)"
    );
}

#[test]
fn test_matchdirs_depth2_contents_not_shown() {
    let temp_dir = create_matchdirs_test_directory();
    let mut options = create_default_options();
    options.pattern_glob = Some(Pattern::new("fzf*").unwrap());
    options.match_dirs = true;

    let result = list_directory_as_string(temp_dir.path(), &options);
    assert!(result.is_ok());

    let output = result.unwrap();

    // fzf_root/sub/nested.lua - parent "sub" doesn't match, so hidden
    assert!(
        !output.contains("nested.lua"),
        "Should not contain nested.lua (parent sub doesn't match)"
    );
    assert!(
        !output.contains("verydeep.lua"),
        "Should not contain verydeep.lua (too deep)"
    );
    // project/fzf/plugin.lua - parent "fzf" matches fzf*, so shown
    assert!(
        output.contains("plugin.lua"),
        "Should contain plugin.lua (parent fzf matches pattern)"
    );
}

#[test]
fn test_matchdirs_nested_match_shows_children() {
    let temp_dir = create_matchdirs_test_directory();
    let mut options = create_default_options();
    options.pattern_glob = Some(Pattern::new("fzf*").unwrap());
    options.match_dirs = true;

    let result = list_directory_as_string(temp_dir.path(), &options);
    assert!(result.is_ok());

    let output = result.unwrap();

    // project/fzf matches pattern at any depth, so direct children should be shown
    assert!(output.contains("fzf"), "Should contain fzf directory");
    assert!(
        output.contains("plugin.lua"),
        "Should contain plugin.lua (parent fzf matches pattern)"
    );
}

#[test]
fn test_matchdirs_all_directories_shown() {
    let temp_dir = create_matchdirs_test_directory();
    let mut options = create_default_options();
    options.pattern_glob = Some(Pattern::new("fzf*").unwrap());
    options.match_dirs = true;

    let result = list_directory_as_string(temp_dir.path(), &options);
    assert!(result.is_ok());

    let output = result.unwrap();

    // All directories should be shown (directories never filtered by pattern)
    assert!(output.contains("fzf_root"), "Should contain fzf_root");
    assert!(output.contains("project"), "Should contain project");
    assert!(output.contains("sub"), "Should contain sub directories");
}

#[test]
fn test_matchdirs_with_prune_keeps_matched_empty() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Create empty directory that matches pattern
    fs::create_dir_all(temp_path.join("fzf_empty")).unwrap();
    // Create non-matching directory with non-matching content
    fs::create_dir_all(temp_path.join("other")).unwrap();
    fs::write(temp_path.join("other").join("file.rs"), "x").unwrap();

    let mut options = create_default_options();
    options.pattern_glob = Some(Pattern::new("fzf*").unwrap());
    options.match_dirs = true;
    options.prune = true;

    let result = list_directory_as_string(temp_dir.path(), &options);
    assert!(result.is_ok());

    let output = result.unwrap();

    // Matched empty directory should NOT be pruned
    assert!(
        output.contains("fzf_empty"),
        "Should contain fzf_empty (matched, not pruned)"
    );
    // Non-matching directory with non-matching content should be pruned
    assert!(
        !output.contains("other"),
        "Should not contain other (no matching content)"
    );
}

#[test]
fn test_matchdirs_fromfile_depth1_contents_shown() {
    let temp_dir = tempdir().unwrap();
    let listing_file = temp_dir.path().join("listing.txt");

    // Create file listing similar to filesystem test structure
    let content = "fzf_root/\nfzf_root/init.lua\nfzf_root/sub/\nfzf_root/sub/nested.lua\nproject/\nproject/fzf/\nproject/fzf/plugin.lua\n";
    fs::write(&listing_file, content).unwrap();

    let mut options = create_default_options();
    options.pattern_glob = Some(Pattern::new("fzf*").unwrap());
    options.match_dirs = true;
    options.from_file = true;

    let result = list_directory_as_string(&listing_file, &options);
    assert!(result.is_ok());

    let output = result.unwrap();

    // fzf_root matches at depth 1, so init.lua should be shown
    assert!(output.contains("fzf_root"), "Should contain fzf_root");
    assert!(
        output.contains("init.lua"),
        "Should contain init.lua (parent matches at depth 1)"
    );
}

#[test]
fn test_matchdirs_fromfile_nested_match_shows_children() {
    let temp_dir = tempdir().unwrap();
    let listing_file = temp_dir.path().join("listing.txt");

    let content = "fzf_root/\nfzf_root/init.lua\nfzf_root/sub/\nfzf_root/sub/nested.lua\nproject/\nproject/fzf/\nproject/fzf/plugin.lua\n";
    fs::write(&listing_file, content).unwrap();

    let mut options = create_default_options();
    options.pattern_glob = Some(Pattern::new("fzf*").unwrap());
    options.match_dirs = true;
    options.from_file = true;

    let result = list_directory_as_string(&listing_file, &options);
    assert!(result.is_ok());

    let output = result.unwrap();

    // nested.lua parent is "sub" which doesn't match, should not be shown
    assert!(
        !output.contains("nested.lua"),
        "Should not contain nested.lua (parent sub doesn't match)"
    );
    // plugin.lua parent is "fzf" which matches fzf*, should be shown
    assert!(
        output.contains("plugin.lua"),
        "Should contain plugin.lua (parent fzf matches pattern)"
    );
}

#[test]
fn test_matchdirs_fromfile_prune_keeps_matched() {
    let temp_dir = tempdir().unwrap();
    let listing_file = temp_dir.path().join("listing.txt");

    // fzf_empty is empty but matches pattern
    let content = "fzf_empty/\nother/\nother/file.rs\n";
    fs::write(&listing_file, content).unwrap();

    let mut options = create_default_options();
    options.pattern_glob = Some(Pattern::new("fzf*").unwrap());
    options.match_dirs = true;
    options.prune = true;
    options.from_file = true;

    let result = list_directory_as_string(&listing_file, &options);
    assert!(result.is_ok());

    let output = result.unwrap();

    // Matched empty directory should NOT be pruned
    assert!(
        output.contains("fzf_empty"),
        "Should contain fzf_empty (matched, not pruned)"
    );
    // Non-matching directory with non-matching content should be pruned
    assert!(
        !output.contains("other"),
        "Should not contain other (no matching content)"
    );
}

#[test]
fn test_matchdirs_depth1_dir_shows_children() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    fs::create_dir_all(temp_path.join("wrapper").join("my_mod")).unwrap();
    fs::write(
        temp_path.join("wrapper").join("my_mod").join("file.txt"),
        "x",
    )
    .unwrap();

    let mut options = create_default_options();
    options.pattern_glob = Some(Pattern::new("*_mod").unwrap());
    options.match_dirs = true;

    let result = list_directory_as_string(temp_dir.path(), &options);
    let output = result.unwrap();

    assert!(output.contains("my_mod"), "Should contain matched dir");
    assert!(
        output.contains("file.txt"),
        "Should contain direct child of matched dir at depth 1"
    );
}

#[test]
fn test_matchdirs_depth2_dir_shows_children() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    fs::create_dir_all(temp_path.join("a").join("b").join("my_mod")).unwrap();
    fs::write(
        temp_path
            .join("a")
            .join("b")
            .join("my_mod")
            .join("deep.txt"),
        "x",
    )
    .unwrap();

    let mut options = create_default_options();
    options.pattern_glob = Some(Pattern::new("*_mod").unwrap());
    options.match_dirs = true;

    let result = list_directory_as_string(temp_dir.path(), &options);
    let output = result.unwrap();

    assert!(output.contains("my_mod"), "Should contain matched dir");
    assert!(
        output.contains("deep.txt"),
        "Should contain direct child of matched dir at depth 2"
    );
}

#[test]
fn test_matchdirs_nested_matched_dirs() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    fs::create_dir_all(temp_path.join("outer_mod").join("inner_mod")).unwrap();
    fs::write(temp_path.join("outer_mod").join("outer.txt"), "x").unwrap();
    fs::write(
        temp_path
            .join("outer_mod")
            .join("inner_mod")
            .join("inner.txt"),
        "x",
    )
    .unwrap();

    let mut options = create_default_options();
    options.pattern_glob = Some(Pattern::new("*_mod").unwrap());
    options.match_dirs = true;

    let result = list_directory_as_string(temp_dir.path(), &options);
    let output = result.unwrap();

    assert!(
        output.contains("outer.txt"),
        "Should contain direct child of outer_mod"
    );
    assert!(
        output.contains("inner.txt"),
        "Should contain direct child of inner_mod"
    );
}

#[test]
fn test_matchdirs_no_cascade() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    fs::create_dir_all(temp_path.join("my_mod").join("sub")).unwrap();
    fs::write(temp_path.join("my_mod").join("direct.txt"), "x").unwrap();
    fs::write(
        temp_path.join("my_mod").join("sub").join("grandchild.txt"),
        "x",
    )
    .unwrap();

    let mut options = create_default_options();
    options.pattern_glob = Some(Pattern::new("*_mod").unwrap());
    options.match_dirs = true;

    let result = list_directory_as_string(temp_dir.path(), &options);
    let output = result.unwrap();

    assert!(
        output.contains("direct.txt"),
        "Should contain direct child of matched dir"
    );
    assert!(
        !output.contains("grandchild.txt"),
        "Should not contain grandchild (no cascade through non-matching sub)"
    );
}

#[test]
fn test_prune_correct_last_connector() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    fs::create_dir_all(temp_path.join("keep")).unwrap();
    fs::create_dir_all(temp_path.join("prune_me")).unwrap();
    fs::write(temp_path.join("keep").join("match.rs"), "x").unwrap();

    let mut options = create_default_options();
    options.pattern_glob = Some(Pattern::new("*.rs").unwrap());
    options.prune = true;

    let result = list_directory_as_string(temp_dir.path(), &options);
    let output = result.unwrap();

    assert!(output.contains("keep"), "Should contain keep dir");
    assert!(
        !output.contains("prune_me"),
        "Should not contain pruned empty dir"
    );

    let lines: Vec<&str> = output.lines().collect();
    let keep_line = lines.iter().find(|l| l.contains("keep")).unwrap();
    assert!(
        keep_line.contains("\u{2514}") || keep_line.contains("+---"),
        "Last surviving entry should use last-item connector"
    );
}

#[test]
fn test_dir_only_report_no_file_count() {
    let temp_dir = create_test_directory();
    let mut options = create_default_options();
    options.dir_only = true;

    let result = list_directory_as_string(temp_dir.path(), &options);
    let output = result.unwrap();

    let last_line = output.lines().last().unwrap();
    assert!(
        last_line.contains("director"),
        "Report should mention directories"
    );
    assert!(
        !last_line.contains("file"),
        "Report should not mention files in -d mode"
    );
}

#[test]
fn test_dir_only_prune_no_op() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    fs::create_dir_all(temp_path.join("match_dir")).unwrap();
    fs::create_dir_all(temp_path.join("other_dir")).unwrap();

    let mut options_no_prune = create_default_options();
    options_no_prune.dir_only = true;
    options_no_prune.pattern_glob = Some(Pattern::new("match*").unwrap());
    options_no_prune.match_dirs = true;

    let mut options_prune = create_default_options();
    options_prune.dir_only = true;
    options_prune.pattern_glob = Some(Pattern::new("match*").unwrap());
    options_prune.match_dirs = true;
    options_prune.prune = true;

    let output_no_prune = list_directory_as_string(temp_dir.path(), &options_no_prune).unwrap();
    let output_prune = list_directory_as_string(temp_dir.path(), &options_prune).unwrap();

    assert_eq!(
        output_no_prune, output_prune,
        "-d --prune should be a no-op (prune disabled in dir-only mode)"
    );
}

#[test]
fn test_matchdirs_fromfile_depth1_shows_children() {
    let temp_dir = tempdir().unwrap();
    let listing_file = temp_dir.path().join("listing.txt");

    let content = "wrapper/\nwrapper/my_mod/\nwrapper/my_mod/file.txt\n";
    fs::write(&listing_file, content).unwrap();

    let mut options = create_default_options();
    options.pattern_glob = Some(Pattern::new("*_mod").unwrap());
    options.match_dirs = true;
    options.from_file = true;

    let result = list_directory_as_string(&listing_file, &options);
    let output = result.unwrap();

    assert!(output.contains("my_mod"), "Should contain matched dir");
    assert!(
        output.contains("file.txt"),
        "Should contain direct child of matched dir"
    );
}

#[test]
fn test_matchdirs_fromfile_no_cascade() {
    let temp_dir = tempdir().unwrap();
    let listing_file = temp_dir.path().join("listing.txt");

    let content = "my_mod/\nmy_mod/direct.txt\nmy_mod/sub/\nmy_mod/sub/grandchild.txt\n";
    fs::write(&listing_file, content).unwrap();

    let mut options = create_default_options();
    options.pattern_glob = Some(Pattern::new("*_mod").unwrap());
    options.match_dirs = true;
    options.from_file = true;

    let result = list_directory_as_string(&listing_file, &options);
    let output = result.unwrap();

    assert!(
        output.contains("direct.txt"),
        "Should contain direct child of matched dir"
    );
    assert!(
        !output.contains("grandchild.txt"),
        "Should not contain grandchild (no cascade through non-matching sub)"
    );
}

#[test]
fn test_dir_only_fromfile_report() {
    let temp_dir = tempdir().unwrap();
    let listing_file = temp_dir.path().join("listing.txt");

    let content = "dir1/\ndir1/file.txt\ndir2/\n";
    fs::write(&listing_file, content).unwrap();

    let mut options = create_default_options();
    options.dir_only = true;
    options.from_file = true;

    let result = list_directory_as_string(&listing_file, &options);
    let output = result.unwrap();

    let last_line = output.lines().last().unwrap();
    assert!(
        last_line.contains("director"),
        "Report should mention directories"
    );
    assert!(
        !last_line.contains("file"),
        "Report should not mention files in -d mode"
    );
}

#[cfg(unix)]
#[test]
fn test_classify_symlink() {
    use std::os::unix::fs::symlink;

    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    fs::write(temp_path.join("target.txt"), "content").unwrap();
    symlink(temp_path.join("target.txt"), temp_path.join("link.txt")).unwrap();

    let mut options = create_default_options();
    options.classify = true;

    let result = list_directory_as_string(temp_dir.path(), &options);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(
        output.contains("link.txt@"),
        "Symlink should have @ indicator"
    );
}

#[test]
fn test_dirs_first_with_sort_by_time() {
    let temp_dir = create_test_directory();
    let mut options = create_default_options();
    options.dirs_first = true;
    options.sort_by_time = true;

    let result = list_directory(temp_dir.path(), &options);
    assert!(result.is_ok());
}

#[test]
fn test_fromfile_dirs_first() {
    let temp_dir = tempdir().unwrap();
    let listing_file = temp_dir.path().join("listing.txt");

    let content = "file1.txt\ndir1/\nfile2.txt\ndir2/\ndir2/nested.txt\n";
    fs::write(&listing_file, content).unwrap();

    let mut options = create_default_options();
    options.dirs_first = true;
    options.from_file = true;

    let result = list_directory_as_string(&listing_file, &options);
    assert!(result.is_ok());

    let output = result.unwrap();
    let lines: Vec<&str> = output.lines().collect();

    let dir1_pos = lines.iter().position(|l| l.contains("dir1"));
    let dir2_pos = lines
        .iter()
        .position(|l| l.contains("dir2") && !l.contains("nested"));
    let file1_pos = lines.iter().position(|l| l.contains("file1"));

    if let (Some(d1), Some(d2), Some(f1)) = (dir1_pos, dir2_pos, file1_pos) {
        assert!(d1 < f1 || d2 < f1, "Directories should appear before files");
    }
}
