pub struct TreeOptions {
    pub all_files: bool,
    pub level: Option<i32>,
    pub full_path: bool,
    pub dir_only: bool,
    pub no_indent: bool,
    pub print_size: bool,
}

pub mod colors;
pub mod display;
pub mod options;
pub mod traversal;
pub mod utils;
