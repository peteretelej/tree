use rust_tree::rust_tree::fromfile::{
    parse_tar_line, parse_tar_listing, parse_tar_simple_line, parse_tar_verbose_line,
    parse_zip_listing, parse_zip_simple_line, parse_zip_verbose_line,
};

#[test]
fn test_parse_tar_verbose_line() {
    // Test directory entry
    let dir_line = "drwxr-xr-x user/group 0 2023-01-01 12:00 src/";
    let entry = parse_tar_verbose_line(dir_line).unwrap();
    assert_eq!(entry.path, "src");
    assert!(entry.is_dir);
    assert_eq!(entry.size, Some(0));

    // Test file entry
    let file_line = "-rw-r--r-- user/group 123 2023-01-01 12:00 src/main.rs";
    let entry = parse_tar_verbose_line(file_line).unwrap();
    assert_eq!(entry.path, "src/main.rs");
    assert!(!entry.is_dir);
    assert_eq!(entry.size, Some(123));

    // Test symlink entry
    let link_line = "lrwxrwxrwx user/group 11 2023-01-01 12:00 symlink";
    let entry = parse_tar_verbose_line(link_line).unwrap();
    assert_eq!(entry.path, "symlink");
    assert!(!entry.is_dir);
    assert_eq!(entry.size, Some(11));
}

#[test]
fn test_parse_tar_verbose_line_invalid() {
    // Test malformed lines
    assert!(parse_tar_verbose_line("").is_none());
    assert!(parse_tar_verbose_line("drwx").is_none());
    assert!(parse_tar_verbose_line("not-a-permission").is_none());
    assert!(parse_tar_verbose_line("drwxr-xr-x user").is_none());
}

#[test]
fn test_parse_tar_simple_line() {
    // Test directory
    let dir_line = "src/";
    let entry = parse_tar_simple_line(dir_line).unwrap();
    assert_eq!(entry.path, "src");
    assert!(entry.is_dir);
    assert!(entry.size.is_none());

    // Test file
    let file_line = "src/main.rs";
    let entry = parse_tar_simple_line(file_line).unwrap();
    assert_eq!(entry.path, "src/main.rs");
    assert!(!entry.is_dir);
    assert!(entry.size.is_none());

    // Test empty line
    assert!(parse_tar_simple_line("").is_none());
}

#[test]
fn test_parse_tar_line() {
    // Test verbose format detection
    let verbose_line = "drwxr-xr-x user/group 0 2023-01-01 12:00 src/";
    let entry = parse_tar_line(verbose_line).unwrap();
    assert_eq!(entry.path, "src");
    assert!(entry.is_dir);

    // Test simple format detection
    let simple_line = "src/main.rs";
    let entry = parse_tar_line(simple_line).unwrap();
    assert_eq!(entry.path, "src/main.rs");
    assert!(!entry.is_dir);

    // Test empty line
    assert!(parse_tar_line("").is_none());
    assert!(parse_tar_line("   ").is_none());
}

#[test]
fn test_parse_tar_listing() {
    let lines = vec![
        "drwxr-xr-x user/group 0 2023-01-01 12:00 src/".to_string(),
        "-rw-r--r-- user/group 123 2023-01-01 12:00 src/main.rs".to_string(),
        "-rw-r--r-- user/group 456 2023-01-01 12:00 README.md".to_string(),
    ];

    let entries = parse_tar_listing(lines);
    assert!(!entries.is_empty());

    // Check for src directory
    let src_dir = entries.iter().find(|e| e.path == "src");
    assert!(src_dir.is_some());
    assert!(src_dir.unwrap().is_dir);

    // Check for main.rs file
    let main_file = entries.iter().find(|e| e.path == "src/main.rs");
    assert!(main_file.is_some());
    assert!(!main_file.unwrap().is_dir);

    // Check for README.md file
    let readme_file = entries.iter().find(|e| e.path == "README.md");
    assert!(readme_file.is_some());
    assert!(!readme_file.unwrap().is_dir);
}

#[test]
fn test_parse_zip_simple_line() {
    // Test standard zip listing line
    let zip_line = "        123  2025-07-26 19:41   src/main.rs";
    let entry = parse_zip_simple_line(zip_line).unwrap();
    assert_eq!(entry.path, "src/main.rs");
    assert!(!entry.is_dir);
    assert_eq!(entry.size, Some(123));

    // Test directory entry
    let dir_line = "          0  2025-07-26 19:41   src/";
    let entry = parse_zip_simple_line(dir_line).unwrap();
    assert_eq!(entry.path, "src");
    assert!(entry.is_dir);
    assert_eq!(entry.size, Some(0));

    // Test file with spaces in name
    let space_line = "        456  2025-07-26 19:41   file with spaces.txt";
    let entry = parse_zip_simple_line(space_line).unwrap();
    assert_eq!(entry.path, "file with spaces.txt");
    assert!(!entry.is_dir);
    assert_eq!(entry.size, Some(456));
}

#[test]
fn test_parse_zip_verbose_line() {
    // Test verbose zip listing line
    let verbose_line = "       123  Stored      123   0% 2025-07-26 19:41 abc12345  src/main.rs";
    let entry = parse_zip_verbose_line(verbose_line).unwrap();
    assert_eq!(entry.path, "src/main.rs");
    assert!(!entry.is_dir);
    assert_eq!(entry.size, Some(123));

    // Test deflated entry
    let deflated_line = "       456  Deflated    234  49% 2025-07-26 19:41 def67890  README.md";
    let entry = parse_zip_verbose_line(deflated_line).unwrap();
    assert_eq!(entry.path, "README.md");
    assert!(!entry.is_dir);
    assert_eq!(entry.size, Some(456));

    // Test directory
    let dir_line = "         0  Stored        0   0% 2025-07-26 19:41 12345678  src/";
    let entry = parse_zip_verbose_line(dir_line).unwrap();
    assert_eq!(entry.path, "src");
    assert!(entry.is_dir);
    assert_eq!(entry.size, Some(0));
}

#[test]
fn test_parse_zip_listing() {
    let lines = vec![
        "Archive:  test.zip".to_string(),
        "  Length      Date    Time    Name".to_string(),
        "---------  ---------- -----   ----".to_string(),
        "        0  2025-07-26 19:41   src/".to_string(),
        "      123  2025-07-26 19:41   src/main.rs".to_string(),
        "      456  2025-07-26 19:41   README.md".to_string(),
        "---------                     -------".to_string(),
        "      579                     3 files".to_string(),
    ];

    let entries = parse_zip_listing(lines);
    assert!(!entries.is_empty());

    // Check for src directory and parent directory creation
    let src_dir = entries.iter().find(|e| e.path == "src");
    assert!(src_dir.is_some());
    assert!(src_dir.unwrap().is_dir);

    // Check for main.rs file
    let main_file = entries.iter().find(|e| e.path == "src/main.rs");
    assert!(main_file.is_some());
    assert!(!main_file.unwrap().is_dir);
}
