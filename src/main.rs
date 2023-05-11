use clap::{App, Arg};
use glob::Pattern;
use std::option::Option;

use rust_tree::rust_tree::options::TreeOptions;
use rust_tree::rust_tree::traversal::list_directory;

fn main() {
    let matches = App::new("tree")
        .arg(Arg::new("directory").index(1).required(false))
        .arg(
            Arg::new("all_files")
                .short('a')
                .help("All files are printed. By default tree does not print hidden files."),
        )
        .arg(
            Arg::new("level")
                .short('L')
                .takes_value(true)
                .help("Max display depth of the directory tree."),
        )
        .arg(
            Arg::new("pattern")
            .short('P')
            .takes_value(true)
            .help("List only those files that match the wild-card pattern. Note: you must use the -a option to also consider those files beginning with a dot '.' for matching."),
        )
        .arg(Arg::new("full_path").short('f').help("Prints the full path prefix for each file."),)
        .arg(Arg::new("dir_only").short('d').help("List directories only."),)
        .arg(Arg::new("no_indent").short('i').help("Makes tree not print the indentation lines, useful when used in conjunction with the -f option."),)
        .arg(Arg::new("print_size").short('s').help("Print the size of each file in bytes along with the name."),)
        .arg(Arg::new("human_readable").short('h').help("Print the size of each file but in a more human readable way, e.g. appending a size letter for kilobytes (K), megabytes (M), gigabytes (G), and so forth."),)
        .arg(
            Arg::new("color")
                .short('C')
                .help("Turn colorization on using built-in color defaults."),
        )
        .arg(
            Arg::new("no_color")
                .short('n')
                .help("Turn colorization off, overridden by -C."),
        )
        .get_matches();

    let path = matches.value_of("directory").unwrap_or(".");
    let level = matches
        .value_of("level")
        .and_then(|l| l.parse::<i32>().ok());
    let pattern_glob: Option<Pattern> = matches.value_of("pattern").map(|pattern| {
        Pattern::new(pattern).unwrap_or_else(|_| {
            eprintln!("Error: Invalid glob pattern.");
            std::process::exit(1);
        })
    });

    let options = TreeOptions {
        all_files: matches.is_present("all_files"),
        level,
        full_path: matches.is_present("full_path"),
        dir_only: matches.is_present("dir_only"),
        no_indent: matches.is_present("no_indent"),
        print_size: matches.is_present("print_size"),
        human_readable: matches.is_present("human_readable"),
        pattern_glob,
        color: matches.is_present("color"),
        no_color: matches.is_present("no_color"),
    };

    if let Err(e) = list_directory(path, &options) {
        eprintln!("Error: {}", e);
    }
}
