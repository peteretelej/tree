use clap::{App, Arg};

fn get_test_app() -> App<'static> {
    App::new("tree")
        .arg(Arg::new("directory").index(1).required(false))
        .arg(
            Arg::new("all_files")
                .short('a')
                .long("all")
                .help("All files are printed. By default tree does not print hidden files."),
        )
        .arg(
            Arg::new("level")
                .short('L')
                .long("level")
                .takes_value(true)
                .required(false)
                .validator(|s| match s.parse::<i32>() {
                    Ok(n) if n >= 0 => Ok(()),
                    _ => Err(String::from("Level must be a non-negative number")),
                })
                .help("Max display depth of the directory tree."),
        )
        .arg(
            Arg::new("pattern")
                .short('P')
                .long("pattern")
                .takes_value(true)
                .required(false)
                .require_equals(true)
                .help("List only those files that match the wild-card pattern."),
        )
        .arg(
            Arg::new("full_path")
                .short('f')
                .long("full-path")
                .help("Prints the full path prefix for each file."),
        )
        .arg(
            Arg::new("dir_only")
                .short('d')
                .long("directories")
                .help("List directories only."),
        )
        .arg(
            Arg::new("no_indent")
                .short('i')
                .long("no-indent")
                .help("Makes tree not print the indentation lines."),
        )
        .arg(
            Arg::new("print_size")
                .short('s')
                .long("size")
                .help("Print the size of each file in bytes along with the name."),
        )
        .arg(
            Arg::new("human_readable")
                .short('h')
                .long("human-readable")
                .help("Print the size of each file in a more human readable way."),
        )
        .arg(
            Arg::new("color")
                .short('C')
                .long("color")
                .help("Turn colorization on."),
        )
        .arg(
            Arg::new("no_color")
                .short('n')
                .long("no-color")
                .help("Turn colorization off."),
        )
        .arg(
            Arg::new("ascii")
                .short('A')
                .long("ascii")
                .help("Use ASCII characters for tree display."),
        )
}

#[test]
fn test_short_flags() {
    let matches = get_test_app()
        .try_get_matches_from(vec!["tree", "-a", "-L", "2", "-d"])
        .unwrap();

    assert!(matches.is_present("all_files"));
    assert_eq!(matches.value_of("level"), Some("2"));
    assert!(matches.is_present("dir_only"));
}

#[test]
fn test_long_flags() {
    let matches = get_test_app()
        .try_get_matches_from(vec!["tree", "--all", "--level=2", "--directories"])
        .unwrap();

    assert!(matches.is_present("all_files"));
    assert_eq!(matches.value_of("level"), Some("2"));
    assert!(matches.is_present("dir_only"));
}

#[test]
fn test_mixed_flags() {
    let matches = get_test_app()
        .try_get_matches_from(vec!["tree", "-a", "--level=2", "-d", "--pattern=*.rs"])
        .unwrap();

    assert!(matches.is_present("all_files"));
    assert_eq!(matches.value_of("level"), Some("2"));
    assert!(matches.is_present("dir_only"));
    assert_eq!(matches.value_of("pattern"), Some("*.rs"));
}

#[test]
fn test_directory_argument() {
    let matches = get_test_app()
        .try_get_matches_from(vec!["tree", "test_dir", "--all"])
        .unwrap();

    assert_eq!(matches.value_of("directory"), Some("test_dir"));
    assert!(matches.is_present("all_files"));
}

#[test]
fn test_invalid_level_value() {
    let result = get_test_app().try_get_matches_from(vec!["tree", "--level=invalid"]);
    assert!(result.is_err(), "Invalid level value should be rejected");

    let result = get_test_app().try_get_matches_from(vec!["tree", "--level=abc"]);
    assert!(
        result.is_err(),
        "Non-numeric level value should be rejected"
    );
}

#[test]
fn test_display_options() {
    let matches = get_test_app()
        .try_get_matches_from(vec![
            "tree",
            "--no-indent",
            "--size",
            "--human-readable",
            "--color",
            "--ascii",
        ])
        .unwrap();

    assert!(matches.is_present("no_indent"));
    assert!(matches.is_present("print_size"));
    assert!(matches.is_present("human_readable"));
    assert!(matches.is_present("color"));
    assert!(matches.is_present("ascii"));
}

#[test]
fn test_color_precedence() {
    // Test that --no-color is overridden by --color when both are present
    let matches = get_test_app()
        .try_get_matches_from(vec!["tree", "--no-color", "--color"])
        .unwrap();

    assert!(matches.is_present("color"));
    assert!(matches.is_present("no_color"));
}

#[test]
fn test_multiple_values() {
    let matches = get_test_app()
        .try_get_matches_from(vec!["tree", "--pattern=*.rs", "--level=3", "src/dir"])
        .unwrap();

    assert_eq!(matches.value_of("pattern"), Some("*.rs"));
    assert_eq!(matches.value_of("level"), Some("3"));
    assert_eq!(matches.value_of("directory"), Some("src/dir"));
}

#[test]
fn test_size_related_flags() {
    // Test interaction between --size and --human-readable
    let matches = get_test_app()
        .try_get_matches_from(vec!["tree", "--size", "--human-readable"])
        .unwrap();

    assert!(matches.is_present("print_size"));
    assert!(matches.is_present("human_readable"));
}

#[test]
fn test_empty_pattern() {
    // Test that --pattern without a value is rejected
    let result = get_test_app().try_get_matches_from(vec!["tree", "--pattern"]);
    assert!(result.is_err(), "Empty pattern should be rejected");

    // Test that -P without a value is rejected
    let result = get_test_app().try_get_matches_from(vec!["tree", "-P"]);
    assert!(result.is_err(), "Empty pattern should be rejected");
}

#[test]
fn test_zero_level() {
    let matches = get_test_app()
        .try_get_matches_from(vec!["tree", "--level=0"])
        .unwrap();

    assert_eq!(matches.value_of("level"), Some("0"));
}

#[test]
fn test_negative_level() {
    let result = get_test_app().try_get_matches_from(vec!["tree", "--level=-1"]);
    assert!(result.is_err(), "Negative level should be rejected");
}
