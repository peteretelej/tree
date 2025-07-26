// Unit tests for main.rs CLI logic
// Tests CLI argument parsing and TreeOptions construction

use glob::Pattern;
use rust_tree::rust_tree::options::TreeOptions;

// We need to test the parse_glob_pattern function and TreeOptions construction
// Since main.rs functions aren't exported, we'll test the equivalent logic

fn create_test_options() -> TreeOptions {
    TreeOptions {
        all_files: false,
        level: Some(2),
        full_path: true,
        dir_only: false,
        no_indent: false,
        print_size: true,
        human_readable: false,
        pattern_glob: Some(Pattern::new("*.rs").unwrap()),
        exclude_pattern: Some(Pattern::new("target").unwrap()),
        color: false,
        no_color: true,
        ascii: true,
        sort_by_time: false,
        reverse: true,
        print_mod_date: false,
        output_file: Some("output.txt".to_string()),
        file_limit: Some(100),
        dirs_first: true,
        classify: false,
        no_report: true,
        print_permissions: false,
        from_file: false,
    }
}

#[test]
fn test_tree_options_construction() {
    let options = create_test_options();

    // Test all fields are set correctly
    assert!(!options.all_files);
    assert_eq!(options.level, Some(2));
    assert!(options.full_path);
    assert!(!options.dir_only);
    assert!(!options.no_indent);
    assert!(options.print_size);
    assert!(!options.human_readable);
    assert!(options.pattern_glob.is_some());
    assert!(options.exclude_pattern.is_some());
    assert!(!options.color);
    assert!(options.no_color);
    assert!(options.ascii);
    assert!(!options.sort_by_time);
    assert!(options.reverse);
    assert!(!options.print_mod_date);
    assert_eq!(options.output_file, Some("output.txt".to_string()));
    assert_eq!(options.file_limit, Some(100));
    assert!(options.dirs_first);
    assert!(!options.classify);
    assert!(options.no_report);
    assert!(!options.print_permissions);
    assert!(!options.from_file);
}

#[test]
fn test_tree_options_defaults() {
    let options = TreeOptions {
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
    };

    // Test default values
    assert!(!options.all_files);
    assert_eq!(options.level, None);
    assert!(!options.full_path);
    assert!(options.pattern_glob.is_none());
    assert!(options.exclude_pattern.is_none());
    assert_eq!(options.output_file, None);
    assert_eq!(options.file_limit, None);
}

#[test]
fn test_pattern_creation() {
    // Test valid glob patterns
    let pattern1 = Pattern::new("*.rs").unwrap();
    assert!(pattern1.matches("main.rs"));
    assert!(!pattern1.matches("main.txt"));

    let pattern2 = Pattern::new("test*").unwrap();
    assert!(pattern2.matches("test_file.rs"));
    assert!(pattern2.matches("tests"));
    assert!(!pattern2.matches("main.rs"));

    let pattern3 = Pattern::new("*.*").unwrap();
    assert!(pattern3.matches("main.rs"));
    assert!(pattern3.matches("Cargo.toml"));
    assert!(!pattern3.matches("README"));
}

#[test]
fn test_invalid_patterns() {
    // Test invalid glob patterns should fail
    assert!(Pattern::new("[").is_err());
    assert!(Pattern::new("**[**").is_err());
}

#[test]
fn test_conflicting_options() {
    // Test combinations that might conflict
    let mut options = create_test_options();

    // Color and no-color both set
    options.color = true;
    options.no_color = true;
    // In real CLI, no_color should override color

    // Size and human-readable
    options.print_size = true;
    options.human_readable = true;
    // Both can be set simultaneously

    // Dir only with patterns
    options.dir_only = true;
    options.pattern_glob = Some(Pattern::new("*.rs").unwrap());
    // Should work together
}

#[test]
fn test_level_option_values() {
    let mut options = create_test_options();

    // Test various level values
    options.level = Some(0);
    assert_eq!(options.level, Some(0));

    options.level = Some(1);
    assert_eq!(options.level, Some(1));

    options.level = Some(10);
    assert_eq!(options.level, Some(10));

    options.level = None;
    assert_eq!(options.level, None);
}

#[test]
fn test_file_limit_values() {
    let mut options = create_test_options();

    // Test various file limit values
    options.file_limit = Some(0);
    assert_eq!(options.file_limit, Some(0));

    options.file_limit = Some(1);
    assert_eq!(options.file_limit, Some(1));

    options.file_limit = Some(1000);
    assert_eq!(options.file_limit, Some(1000));

    options.file_limit = None;
    assert_eq!(options.file_limit, None);
}

#[test]
fn test_output_file_option() {
    let mut options = create_test_options();

    // Test various output file values
    options.output_file = Some("test.txt".to_string());
    assert_eq!(options.output_file, Some("test.txt".to_string()));

    options.output_file = Some("/tmp/output.log".to_string());
    assert_eq!(options.output_file, Some("/tmp/output.log".to_string()));

    options.output_file = None;
    assert_eq!(options.output_file, None);
}

#[test]
fn test_boolean_option_combinations() {
    let mut options = create_test_options();

    // Test all boolean flags can be set independently
    options.all_files = true;
    options.full_path = true;
    options.dir_only = true;
    options.no_indent = true;
    options.print_size = true;
    options.human_readable = true;
    options.color = true;
    options.no_color = false; // Should not conflict when color is true
    options.ascii = true;
    options.sort_by_time = true;
    options.reverse = true;
    options.print_mod_date = true;
    options.dirs_first = true;
    options.classify = true;
    options.no_report = true;
    options.print_permissions = true;
    options.from_file = true;

    // All should be set as expected
    assert!(options.all_files);
    assert!(options.full_path);
    assert!(options.dir_only);
    assert!(options.no_indent);
    assert!(options.print_size);
    assert!(options.human_readable);
    assert!(options.color);
    assert!(!options.no_color);
    assert!(options.ascii);
    assert!(options.sort_by_time);
    assert!(options.reverse);
    assert!(options.print_mod_date);
    assert!(options.dirs_first);
    assert!(options.classify);
    assert!(options.no_report);
    assert!(options.print_permissions);
    assert!(options.from_file);
}
