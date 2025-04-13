use clap::Parser;
use glob::Pattern;
use std::option::Option;

use rust_tree::rust_tree::options::TreeOptions;
use rust_tree::rust_tree::traversal::list_directory;

#[derive(Parser)]
#[command(name = "tree")]
#[command(about = "Display directory tree structure")]
struct Cli {
    #[arg(index = 1, default_value = ".")]
    directory: String,

    #[arg(short = 'a', long = "all", help = "All files are printed. By default tree does not print hidden files.")]
    all_files: bool,

    #[arg(short = 'L', long = "level", help = "Max display depth of the directory tree.")]
    level: Option<i32>,

    #[arg(short = 'P', long = "pattern", help = "List only those files that match the wild-card pattern. Note: you must use the -a option to also consider those files beginning with a dot '.' for matching.")]
    pattern: Option<String>,

    #[arg(short = 'I', long = "exclude", help = "Do not list files that match the wild-card pattern.")]
    exclude: Option<String>,

    #[arg(short = 'f', long = "full-path", help = "Prints the full path prefix for each file.")]
    full_path: bool,

    #[arg(short = 'd', long = "directories", help = "List directories only.")]
    dir_only: bool,

    #[arg(short = 'i', long = "no-indent", help = "Makes tree not print the indentation lines, useful when used in conjunction with the -f option.")]
    no_indent: bool,

    #[arg(short = 's', long = "size", help = "Print the size of each file in bytes along with the name.")]
    print_size: bool,

    #[arg(short = 'H', long = "human-readable", help = "Print the size of each file but in a more human readable way, e.g. appending a size letter for kilobytes (K), megabytes (M), gigabytes (G), and so forth.")]
    human_readable: bool,

    #[arg(short = 'C', long = "color", help = "Turn colorization on using built-in color defaults.")]
    color: bool,

    #[arg(short = 'n', long = "no-color", help = "Turn colorization off, overridden by -C.")]
    no_color: bool,

    #[arg(short = 'A', long = "ascii", help = "Use only ASCII characters on tree display.")]
    ascii: bool,
}

fn main() {
    let cli = Cli::parse();
    
    let pattern_glob: Option<Pattern> = cli.pattern.map(|pattern| {
        Pattern::new(&pattern).unwrap_or_else(|_| {
            eprintln!("Error: Invalid glob pattern.");
            std::process::exit(1);
        })
    });

    let exclude_pattern: Option<Pattern> = cli.exclude.map(|pattern| {
        Pattern::new(&pattern).unwrap_or_else(|_| {
            eprintln!("Error: Invalid exclude pattern.");
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
    };

    if let Err(e) = list_directory(&cli.directory, &options) {
        eprintln!("Error: {}", e);
    }
}
