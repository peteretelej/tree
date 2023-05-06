use clap::{App, Arg};
use tree::tree::options::TreeOptions;
use tree::tree::traversal::list_directory;

fn main() {
    let matches = App::new("tree")
        .arg(Arg::new("directory").index(1).required(false))
        .arg(
            Arg::new("all_files")
                .short('a')
                .long("all-files")
                .help("All files are printed. By default tree does not print hidden files."),
        )
        .arg(
            Arg::new("level")
                .short('L')
                .long("level")
                .takes_value(true)
                .help("Max display depth of the directory tree."),
        )
        .arg(Arg::new("full_path").short('f').long("full-path").help("Prints the full path prefix for each file."),)
        .arg(Arg::new("dir_only").short('d').long("dir-only").help("List directories only."),)
        .arg(Arg::new("no_indent").short('i').long("no-indent").help("Makes tree not print the indentation lines, useful when used in conjunction with the -f option."),)
        .arg(Arg::new("print_size").short('s').long("print-size").help("Print the size of each file in bytes along with the name."),)
        .get_matches();

    let path = matches.value_of("directory").unwrap_or(".");
    let level = matches
        .value_of("level")
        .and_then(|l| l.parse::<i32>().ok());

    let options = TreeOptions {
        all_files: matches.is_present("all_files"),
        level,
        full_path: matches.is_present("full_path"), // TODO: implement full_path
        dir_only: matches.is_present("dir_only"),   // TODO: implement dir_only
        no_indent: matches.is_present("no_indent"), // TODO: implement no_indent
        print_size: matches.is_present("print_size"), // TODO: implement print_size
    };

    if let Err(e) = list_directory(path, &options) {
        eprintln!("Error: {}", e);
    }
}
