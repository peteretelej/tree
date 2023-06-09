use glob::Pattern;

pub struct TreeOptions {
    pub all_files: bool,
    pub level: Option<i32>,
    pub full_path: bool,
    pub dir_only: bool,
    pub no_indent: bool,
    pub print_size: bool,
    pub human_readable: bool,
    pub pattern_glob: Option<Pattern>,
    pub color: bool,
    pub no_color: bool,
}
