use clap::Parser;
use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[derive(Parser)]
#[command(name = "tree")]
struct Cli {
    #[arg(index = 1)]
    directory: Option<String>,

    #[arg(short = 'a', long = "all")]
    all_files: bool,

    #[arg(short = 'L', long = "level")]
    level: Option<i32>,

    #[arg(short = 'P', long = "pattern")]
    pattern: Option<String>,

    #[arg(short = 'f', long = "full-path")]
    full_path: bool,

    #[arg(short = 'd', long = "directories")]
    dir_only: bool,

    #[arg(short = 'i', long = "no-indent")]
    no_indent: bool,

    #[arg(short = 's', long = "size")]
    print_size: bool,

    #[arg(short = 'H', long = "human-readable")]
    human_readable: bool,

    #[arg(short = 'C', long = "color")]
    color: bool,

    #[arg(short = 'n', long = "no-color")]
    no_color: bool,

    #[arg(short = 'A', long = "ascii")]
    ascii: bool,

    #[arg(short = 'o', long = "output")]
    output_file: Option<String>,

    #[arg(short = 'D', long = "mod-date")]
    print_mod_date: bool,

    #[arg(short = 't', long = "sort-by-time")]
    sort_by_time: bool,

    #[arg(short = 'r', long = "reverse")]
    reverse: bool,

    #[arg(long = "filelimit")]
    file_limit: Option<u64>,
}

#[test]
fn test_short_flags() {
    let cli = Cli::try_parse_from(["tree", "-a", "-L", "2", "-d"]).unwrap();
    assert!(cli.all_files);
    assert_eq!(cli.level, Some(2));
    assert!(cli.dir_only);
}

#[test]
fn test_long_flags() {
    let cli = Cli::try_parse_from(["tree", "--all", "--level=2", "--directories"]).unwrap();
    assert!(cli.all_files);
    assert_eq!(cli.level, Some(2));
    assert!(cli.dir_only);
}

#[test]
fn test_mixed_flags() {
    let cli = Cli::try_parse_from(["tree", "-a", "--level=2", "-d", "--pattern=*.rs"]).unwrap();
    assert!(cli.all_files);
    assert_eq!(cli.level, Some(2));
    assert!(cli.dir_only);
    assert_eq!(cli.pattern, Some("*.rs".to_string()));
}

#[test]
fn test_directory_argument() {
    let cli = Cli::try_parse_from(["tree", "test_dir", "--all"]).unwrap();
    assert_eq!(cli.directory, Some("test_dir".to_string()));
    assert!(cli.all_files);
}

#[test]
fn test_invalid_level_value() {
    assert!(Cli::try_parse_from(["tree", "--level=invalid"]).is_err());
    assert!(Cli::try_parse_from(["tree", "--level=abc"]).is_err());
}

#[test]
fn test_display_options() {
    let cli = Cli::try_parse_from([
        "tree",
        "--no-indent",
        "--size",
        "--human-readable",
        "--color",
        "--ascii",
    ])
    .unwrap();

    assert!(cli.no_indent);
    assert!(cli.print_size);
    assert!(cli.human_readable);
    assert!(cli.color);
    assert!(cli.ascii);
}

#[test]
fn test_color_precedence() {
    let cli = Cli::try_parse_from(["tree", "--no-color", "--color"]).unwrap();
    assert!(cli.color);
    assert!(cli.no_color);
}

#[test]
fn test_multiple_values() {
    let cli = Cli::try_parse_from(["tree", "--pattern=*.rs", "--level=3", "src/dir"]).unwrap();
    assert_eq!(cli.pattern, Some("*.rs".to_string()));
    assert_eq!(cli.level, Some(3));
    assert_eq!(cli.directory, Some("src/dir".to_string()));
}

#[test]
fn test_size_related_flags() {
    let cli = Cli::try_parse_from(["tree", "--size", "--human-readable"]).unwrap();
    assert!(cli.print_size);
    assert!(cli.human_readable);
}

#[test]
fn test_output_to_file() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary directory for the test content
    let content_dir = tempdir()?;
    let content_dir_path = content_dir.path();

    // Create the output file path in the *parent* of the content directory
    let output_path = content_dir_path.parent().unwrap().join("output.txt");
    let output_path_str = output_path.to_str().unwrap();

    // Create a dummy file inside the content directory
    let dummy_file_path = content_dir_path.join("dummy.txt");
    fs::write(&dummy_file_path, "content")?;

    // Run the `tree` command scanning the content directory, outputting outside
    let mut cmd = Command::cargo_bin("tree")?;
    cmd.arg(content_dir_path.to_str().unwrap())
       .arg("-o")
       .arg(output_path_str)
       .assert()
       .success();

    // Read the contents of the output file
    let output_content = fs::read_to_string(&output_path)?;
    //println!("Debug: Output file content:\n---\n{}\n---", output_content); // Keep for debugging if needed

    // Assert that the output file contains the expected content
    assert!(output_content.contains("dummy.txt"));
    assert!(
        output_content.contains("0 directories, 1 file"),
        "Summary line '0 directories, 1 file' not found in output: {}\n", output_content
    );

    // Clean up the output file manually as it's outside the tempdir scope
    fs::remove_file(&output_path)?;

    // The content directory and its contents are automatically cleaned up when `content_dir` goes out of scope
    Ok(())
}

#[test]
fn test_file_limit() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let root = dir.path();
    let sub_dir = root.join("sub_dir");
    fs::create_dir(&sub_dir)?;
    fs::write(sub_dir.join("file1.txt"), "1")?;
    fs::write(sub_dir.join("file2.txt"), "2")?;
    fs::write(sub_dir.join("file3.txt"), "3")?;
    fs::write(root.join("root_file.txt"), "root")?;

    // --- Test 1: Run without filelimit (output to file) --- 
    let output_path1 = root.parent().unwrap().join("output1.txt");
    let mut cmd1 = Command::cargo_bin("tree")?;
    cmd1.arg(root.to_str().unwrap())
        .arg("-o").arg(&output_path1)
        .assert()
        .success();
    let content1 = fs::read_to_string(&output_path1)?;
    fs::remove_file(&output_path1)?; // Clean up

    println!("Content without --filelimit:\n{}", content1); // Debug print
    assert!(content1.contains("sub_dir"));
    assert!(content1.contains("file1.txt"));
    assert!(content1.contains("file2.txt"));
    assert!(content1.contains("file3.txt"));
    assert!(content1.contains("root_file.txt"));
    // Check summary components separately to avoid newline issues
    assert!(content1.contains("1 directory"), "Test 1 Summary Failed (dir count): Content was\n{}", content1);
    assert!(content1.contains("4 files"), "Test 1 Summary Failed (file count): Content was\n{}", content1);

    // --- Test 2: Run with filelimit = 2 (output to file) --- 
    let output_path2 = root.parent().unwrap().join("output2.txt");
    let mut cmd2 = Command::cargo_bin("tree")?;
    cmd2.arg(root.to_str().unwrap())
        .arg("--filelimit=2")
        .arg("-o").arg(&output_path2)
        .assert()
        .success();
    let content2 = fs::read_to_string(&output_path2)?;
    fs::remove_file(&output_path2)?; // Clean up
    
    println!("Content with --filelimit=2:\n{}", content2); // Debug print
    assert!(content2.contains("sub_dir"));
    assert!(!content2.contains("file1.txt"));
    assert!(!content2.contains("file2.txt"));
    assert!(!content2.contains("file3.txt"));
    assert!(content2.contains("root_file.txt"));
    // Check summary components separately
    assert!(content2.contains("1 directory"), "Test 2 Summary Failed (dir count): Content was\n{}", content2);
    assert!(content2.contains("1 file"), "Test 2 Summary Failed (file count): Content was\n{}", content2);

    Ok(())
}
