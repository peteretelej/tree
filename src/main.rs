use clap::{App, Arg};
use tree::tree::TreeOptions;
use tree::tree::traversal::list_directory;


fn main() {
    let matches = App::new("tree")
        .arg(Arg::new("directory").index(1).required(false))
        .arg(Arg::new("all_files").short('a').long("all-files"))
        .arg(Arg::new("full_path").short('f').long("full-path"))
        .arg(Arg::new("dir_only").short('d').long("dir-only"))
        .arg(Arg::new("no_indent").short('i').long("no-indent"))
        .arg(Arg::new("print_size").short('s').long("print-size"))
        .get_matches();

    let path = matches.value_of("directory").unwrap_or(".");
    
    let options = TreeOptions {
        all_files: matches.is_present("all_files"),
        full_path: matches.is_present("full_path"), // TODO: implement full_path
        dir_only: matches.is_present("dir_only"), // TODO: implement dir_only
        no_indent: matches.is_present("no_indent"), // TODO: implement no_indent
        print_size: matches.is_present("print_size"), // TODO: implement print_size
    };

    if let Err(e) = list_directory(path, &options) {
        eprintln!("Error: {}", e);
    }
}
