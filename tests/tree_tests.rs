use std::collections::HashSet;
use std::fs::{create_dir_all, File};
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
    let stdout_str = String::from_utf8(stdout).expect("Bad parsing");
    stdout_str
}

fn create_test_directory() {
    let base = "tests/test_directory";
    create_dir_all(format!("{}/dir1/dir1_1", base)).unwrap();
    create_dir_all(format!("{}/dir2", base)).unwrap();
    File::create(format!("{}/file1.txt", base)).unwrap();
    File::create(format!("{}/dir1/file2.txt", base)).unwrap();
    File::create(format!("{}/dir2/file3.txt", base)).unwrap();
    File::create(format!("{}/.hidden.txt", base)).unwrap();
}

fn last_line(output: &str) -> &str {
    output.lines().last().unwrap_or("")
}

fn lines_to_set(s: &str) -> HashSet<&str> {
    s.lines().collect()
}

#[test]
fn test_normal() {
    create_test_directory();
    let expected = r#"test_directory
├── dir1
│   ├── dir1_1
│   └── file2.txt
├── dir2
│   └── file3.txt
└── file1.txt

3 directories, 3 files
"#;

    let output = run_cmd(&["tests/test_directory"]);
    assert_eq!(lines_to_set(expected), lines_to_set(&output));
}

#[test]
fn test_max_depth() {
    create_test_directory();
    let expected = r#"test_directory
├── dir1
├── dir2
└── file1.txt

2 directories, 1 files
"#;

    let output = run_cmd(&["-L", "1", "tests/test_directory"]);
    assert_eq!(lines_to_set(expected), lines_to_set(&output));
}

#[test]
fn test_filter_txt_files() {
    create_test_directory();
    let expected = r#"test_directory
├── dir1
│   ├── dir1_1
│   └── file2.txt
├── dir2
│   └── file3.txt
└── file1.txt

3 directories, 3 files
"#;

    let output = run_cmd(&["-P", "*.txt", "tests/test_directory"]);
    assert_eq!(lines_to_set(expected), lines_to_set(&output));
}

#[test]
fn test_normal_summary() {
    create_test_directory();
    let expected = "3 directories, 3 files";

    let output = run_cmd(&["tests/test_directory"]);
    assert_eq!(expected, last_line(&output));
}

#[test]
fn test_max_depth_summary() {
    create_test_directory();
    let expected = "2 directories, 1 files";

    let output = run_cmd(&["-L", "1", "tests/test_directory"]);
    assert_eq!(expected, last_line(&output));
}

#[test]
fn test_filter_txt_files_summary() {
    create_test_directory();
    let expected = "3 directories, 3 files";

    let output = run_cmd(&["-P", "*.txt", "tests/test_directory"]);
    assert_eq!(expected, last_line(&output));
}

#[test]
fn test_hidden_files() {
    create_test_directory();
    let output = run_cmd(&["tests/test_directory"]);

    assert!(
        !output.contains(".hidden.txt"),
        "Hidden files should not be listed without -a flag"
    );

    let output = run_cmd(&["-a", "tests/test_directory"]);

    assert!(
        output.contains(".hidden.txt"),
        "Hidden files should be listed with -a flag"
    );
}
