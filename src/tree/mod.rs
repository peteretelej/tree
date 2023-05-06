pub struct TreeOptions {
    pub all_files: bool,
    pub full_path: bool,
    pub dir_only: bool,
    pub no_indent: bool,
    pub print_size: bool,
}

pub mod options;
pub mod display;
pub mod colors;
pub mod utils;
pub mod traversal;
