use clap::Parser;

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
