// CLI module - handles cli flag parsing

use clap::Parser;
use glob::Pattern;
use std::io;

use crate::rust_tree::options::TreeOptions;
use crate::rust_tree::traversal::list_directory;

// Custom function to validate the glob pattern
pub fn parse_glob_pattern(s: &str) -> Result<Pattern, String> {
    Pattern::new(s).map_err(|e| e.to_string())
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(default_value = ".", help = "The path to the directory to list.")]
    pub path: String,

    #[arg(short = 'a', long = "all", help = "Include hidden files.")]
    pub all_files: bool,

    #[arg(
        short = 'L',
        long = "level",
        help = "Descend only level directories deep."
    )]
    pub level: Option<u32>,

    #[arg(short = 'd', long = "directories", help = "List directories only.")]
    pub dir_only: bool,

    #[arg(
        short = 'i',
        long = "no-indent",
        help = "Turn off file/directory indentation."
    )]
    pub no_indent: bool,

    #[arg(
        short = 's',
        long = "size",
        help = "Print the size of each file in bytes."
    )]
    pub print_size: bool,

    #[arg(
        short = 'H',
        long = "human-readable",
        help = "Print the size in a more human-readable format."
    )]
    pub human_readable: bool,

    #[arg(
        short = 'P',
        long = "pattern",
        help = "List only those files that match the wild-card pattern."
    )]
    pub pattern: Option<String>,

    #[arg(
        short = 'I',
        long = "exclude",
        help = "Do not list files that match the wild-card pattern."
    )]
    pub exclude: Option<String>,

    #[arg(
        short = 'f',
        long = "full-path",
        help = "Prints the full path prefix for each file."
    )]
    pub full_path: bool,

    #[arg(
        short = 'C',
        long = "color",
        help = "Turn colorization on always, using built-in color defaults if the LS_COLORS environment variable is not set. Helpful when piping output to other programs."
    )]
    pub color: bool,

    #[arg(
        short = 'n',
        long = "no-color",
        help = "Turn colorization off always (--no-color overrides --color)."
    )]
    pub no_color: bool,

    #[arg(
        short = 'A',
        long = "ascii",
        help = "Turn on ANSI line graphics hack when printing the indentation lines."
    )]
    pub ascii: bool,

    #[arg(
        short = 't',
        long = "sort-by-time",
        help = "Sort output by last modification time instead of alphabetically."
    )]
    pub sort_by_time: bool,

    #[arg(short = 'r', long = "reverse", help = "Reverse the sort order.")]
    pub reverse: bool,

    #[arg(
        short = 'D',
        long = "mod-date",
        help = "Print the date of last modification."
    )]
    pub print_mod_date: bool,

    #[arg(short = 'o', long = "output", help = "Send output to filename.")]
    pub output_file: Option<String>,

    #[arg(
        long = "filelimit",
        value_name = "#",
        help = "Do not descend directories that contain more than # entries."
    )]
    pub file_limit: Option<u64>,

    #[arg(long = "dirsfirst", help = "List directories before files.")]
    pub dirs_first: bool,

    #[arg(
        short = 'F',
        long = "classify",
        help = "Append indicator (one of */=@|%>) to entries."
    )]
    pub classify: bool,

    #[arg(
        long = "noreport",
        help = "Omits printing of the file and directory report at the end of the tree listing."
    )]
    pub no_report: bool,

    #[arg(short = 'p', help = "Print the protections for each file (unix only).")]
    pub print_permissions: bool,

    #[arg(long = "fromfile", help = "Read listing from file/stdin")]
    pub fromfile: bool,
}

/// Convert CLI arguments to TreeOptions
pub fn cli_to_options(cli: &Cli) -> Result<TreeOptions, String> {
    let pattern_glob: Option<Pattern> = cli
        .pattern
        .as_ref()
        .map(|pattern| parse_glob_pattern(pattern))
        .transpose()?;

    let exclude_pattern: Option<Pattern> = cli
        .exclude
        .as_ref()
        .map(|pattern| parse_glob_pattern(pattern))
        .transpose()?;

    Ok(TreeOptions {
        all_files: cli.all_files,
        level: cli.level,
        full_path: cli.full_path,
        dir_only: cli.dir_only,
        no_indent: cli.no_indent,
        print_size: cli.print_size,
        human_readable: cli.human_readable,
        pattern_glob,
        exclude_pattern,
        color: cli.color,
        no_color: cli.no_color,
        ascii: cli.ascii,
        sort_by_time: cli.sort_by_time,
        reverse: cli.reverse,
        print_mod_date: cli.print_mod_date,
        output_file: cli.output_file.clone(),
        file_limit: cli.file_limit,
        dirs_first: cli.dirs_first,
        classify: cli.classify,
        no_report: cli.no_report,
        print_permissions: cli.print_permissions,
        from_file: cli.fromfile,
    })
}

/// Main CLI execution function
pub fn run_cli() -> io::Result<()> {
    let cli = Cli::parse();
    let options = cli_to_options(&cli).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid pattern: {e}"))
    })?;

    list_directory(&cli.path, &options)
}

/// Execute with provided arguments (useful for testing)
pub fn run_with_args(args: Vec<String>) -> io::Result<()> {
    let cli = Cli::try_parse_from(args)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;

    let options = cli_to_options(&cli).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid pattern: {e}"))
    })?;

    list_directory(&cli.path, &options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_glob_pattern_valid() {
        let result = parse_glob_pattern("*.rs");
        assert!(result.is_ok());

        let pattern = result.unwrap();
        assert!(pattern.matches("main.rs"));
        assert!(!pattern.matches("main.txt"));
    }

    #[test]
    fn test_parse_glob_pattern_invalid() {
        let result = parse_glob_pattern("[");
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_to_options_default() {
        let cli = Cli {
            path: ".".to_string(),
            all_files: false,
            level: None,
            dir_only: false,
            no_indent: false,
            print_size: false,
            human_readable: false,
            pattern: None,
            exclude: None,
            full_path: false,
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
            fromfile: false,
        };

        let options = cli_to_options(&cli).unwrap();
        assert!(!options.all_files);
        assert!(options.level.is_none());
        assert!(!options.dir_only);
        assert!(options.pattern_glob.is_none());
        assert!(options.exclude_pattern.is_none());
    }

    #[test]
    fn test_cli_to_options_with_patterns() {
        let cli = Cli {
            path: ".".to_string(),
            all_files: true,
            level: Some(2),
            dir_only: false,
            no_indent: false,
            print_size: true,
            human_readable: true,
            pattern: Some("*.rs".to_string()),
            exclude: Some("target".to_string()),
            full_path: true,
            color: true,
            no_color: false,
            ascii: true,
            sort_by_time: true,
            reverse: true,
            print_mod_date: true,
            output_file: Some("output.txt".to_string()),
            file_limit: Some(100),
            dirs_first: true,
            classify: true,
            no_report: true,
            print_permissions: true,
            fromfile: true,
        };

        let options = cli_to_options(&cli).unwrap();
        assert!(options.all_files);
        assert_eq!(options.level, Some(2));
        assert!(options.print_size);
        assert!(options.human_readable);
        assert!(options.pattern_glob.is_some());
        assert!(options.exclude_pattern.is_some());
        assert!(options.full_path);
        assert!(options.color);
        assert!(options.ascii);
        assert!(options.sort_by_time);
        assert!(options.reverse);
        assert!(options.print_mod_date);
        assert_eq!(options.output_file, Some("output.txt".to_string()));
        assert_eq!(options.file_limit, Some(100));
        assert!(options.dirs_first);
        assert!(options.classify);
        assert!(options.no_report);
        assert!(options.print_permissions);
        assert!(options.from_file);
    }

    #[test]
    fn test_cli_to_options_invalid_pattern() {
        let cli = Cli {
            path: ".".to_string(),
            all_files: false,
            level: None,
            dir_only: false,
            no_indent: false,
            print_size: false,
            human_readable: false,
            pattern: Some("[".to_string()), // Invalid pattern
            exclude: None,
            full_path: false,
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
            fromfile: false,
        };

        let result = cli_to_options(&cli);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_with_args_basic() {
        // Test basic execution - this should work without error
        let args = vec!["tree".to_string(), "--help".to_string()];
        // This will fail because --help causes clap to exit, but we test the parsing
        let result = run_with_args(args);
        // Help flag causes early exit, so we expect an error here
        assert!(result.is_err());
    }

    #[test]
    fn test_run_with_args_invalid() {
        let args = vec!["tree".to_string(), "--invalid-flag".to_string()];
        let result = run_with_args(args);
        assert!(result.is_err());
    }
}
