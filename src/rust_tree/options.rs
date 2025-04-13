use glob::Pattern;

pub struct TreeOptions {
    pub all_files: bool,
    pub level: Option<u32>,
    pub full_path: bool,
    pub dir_only: bool,
    pub no_indent: bool,
    pub print_size: bool,
    pub human_readable: bool,
    pub pattern_glob: Option<Pattern>,
    pub exclude_pattern: Option<Pattern>,
    pub color: bool,
    pub no_color: bool,
    pub ascii: bool,
    pub sort_by_time: bool,
    pub reverse: bool,
    pub print_mod_date: bool,
    pub output_file: Option<String>,
    pub file_limit: Option<u64>,
    pub dirs_first: bool,
    pub classify: bool,
}
