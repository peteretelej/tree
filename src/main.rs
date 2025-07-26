use clap::Parser;
use glob::Pattern;
use std::io;
use std::option::Option;

use rust_tree::rust_tree::options::TreeOptions;
use rust_tree::rust_tree::traversal::list_directory;
use rust_tree::rust_tree::utils::is_broken_pipe_error;

// Custom function to validate the glob pattern
fn parse_glob_pattern(s: &str) -> Result<Pattern, String> {
    Pattern::new(s).map_err(|e| e.to_string())
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(default_value = ".", help = "The path to the directory to list.")]
    path: String,

    #[arg(short = 'a', long = "all", help = "Include hidden files.")]
    all_files: bool,

    #[arg(
        short = 'L',
        long = "level",
        help = "Descend only level directories deep."
    )]
    level: Option<u32>,

    #[arg(short = 'd', long = "directories", help = "List directories only.")]
    dir_only: bool,

    #[arg(
        short = 'i',
        long = "no-indent",
        help = "Turn off file/directory indentation."
    )]
    no_indent: bool,

    #[arg(
        short = 's',
        long = "size",
        help = "Print the size of each file in bytes."
    )]
    print_size: bool,

    #[arg(
        short = 'H',
        long = "human-readable",
        help = "Print the size in a more human-readable format."
    )]
    human_readable: bool,

    #[arg(
        short = 'P',
        long = "pattern",
        help = "List only those files that match the wild-card pattern."
    )]
    pattern: Option<String>,

    #[arg(
        short = 'I',
        long = "exclude",
        help = "Do not list files that match the wild-card pattern."
    )]
    exclude: Option<String>,

    #[arg(
        short = 'f',
        long = "full-path",
        help = "Prints the full path prefix for each file."
    )]
    full_path: bool,

    #[arg(
        short = 'C',
        long = "color",
        help = "Turn colorization on always, using built-in color defaults if the LS_COLORS environment variable is not set. Helpful when piping output to other programs."
    )]
    color: bool,

    #[arg(
        short = 'n',
        long = "no-color",
        help = "Turn colorization off always (--no-color overrides --color)."
    )]
    no_color: bool,

    #[arg(
        short = 'A',
        long = "ascii",
        help = "Turn on ANSI line graphics hack when printing the indentation lines."
    )]
    ascii: bool,

    #[arg(
        short = 't',
        long = "sort-by-time",
        help = "Sort output by last modification time instead of alphabetically."
    )]
    sort_by_time: bool,

    #[arg(short = 'r', long = "reverse", help = "Reverse the sort order.")]
    reverse: bool,

    #[arg(
        short = 'D',
        long = "mod-date",
        help = "Print the date of last modification."
    )]
    print_mod_date: bool,

    #[arg(short = 'o', long = "output", help = "Send output to filename.")]
    output_file: Option<String>,

    #[arg(
        long = "filelimit",
        value_name = "#",
        help = "Do not descend directories that contain more than # entries."
    )]
    file_limit: Option<u64>,

    #[arg(long = "dirsfirst", help = "List directories before files.")]
    dirs_first: bool,

    #[arg(
        short = 'F',
        long = "classify",
        help = "Append indicator (one of */=@|%>) to entries."
    )]
    classify: bool,

    #[arg(
        long = "noreport",
        help = "Omits printing of the file and directory report at the end of the tree listing."
    )]
    no_report: bool,

    #[arg(short = 'p', help = "Print the protections for each file (unix only).")]
    print_permissions: bool,

    #[arg(long = "fromfile", help = "Read listing from file/stdin")]
    fromfile: bool,
}

fn try_main() -> io::Result<()> {
    let cli = Cli::parse();

    let pattern_glob: Option<Pattern> = cli.pattern.map(|pattern| {
        parse_glob_pattern(&pattern).unwrap_or_else(|e| {
            eprintln!("Error: Invalid pattern: {e}");
            std::process::exit(1);
        })
    });

    let exclude_pattern: Option<Pattern> = cli.exclude.map(|pattern| {
        parse_glob_pattern(&pattern).unwrap_or_else(|e| {
            eprintln!("Error: Invalid exclude pattern: {e}");
            std::process::exit(1);
        })
    });

    let options = TreeOptions {
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
    };

    list_directory(&cli.path, &options)
}

fn main() {
    match try_main() {
        Ok(()) => {}
        Err(err) => {
            if is_broken_pipe_error(&err) {
                // silently terminate for broken pipe to gracefully handle SIGPIPE
                std::process::exit(0);
            } else {
                eprintln!("Error: {err}");
                std::process::exit(1);
            }
        }
    }
}
