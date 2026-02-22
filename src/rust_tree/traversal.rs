use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

// Add import for Unix permissions
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::rust_tree::display::colorize;
use crate::rust_tree::icons::IconManager;
// Conditionally import the permissions formatter only on Unix
#[cfg(unix)]
use crate::rust_tree::display::format_permissions_unix;
use crate::rust_tree::fromfile::{
    build_virtual_tree, parse_file_listing, read_file_listing, FileEntry, VirtualTree,
};
use crate::rust_tree::options::TreeOptions;
use crate::rust_tree::utils::bytes_to_human_readable;

// stdlib date formatting coz chrono is a pain to cross compile
fn format_date(time: SystemTime) -> String {
    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let secs = duration.as_secs();
            let time_parts = (
                (secs / 86400) % 36525, // days since epoch
                ((secs / 3600) % 24),   // hours
                ((secs / 60) % 60),     // minutes
                (secs % 60),            // seconds
            );

            // Start with Unix epoch (1970-01-01) and add days
            let mut year = 1970;
            let mut month = 1;
            let mut day = 1;
            let mut days_left = time_parts.0;

            // Simple date calculation - consider leap years, etc.
            while days_left > 0 {
                let days_in_year = if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                    366
                } else {
                    365
                };
                if days_left >= days_in_year {
                    days_left -= days_in_year;
                    year += 1;
                } else {
                    let days_in_month = match month {
                        2 => {
                            if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                                29
                            } else {
                                28
                            }
                        }
                        4 | 6 | 9 | 11 => 30,
                        _ => 31,
                    };

                    if days_left >= days_in_month {
                        days_left -= days_in_month;
                        month += 1;
                        if month > 12 {
                            month = 1;
                            year += 1;
                        }
                    } else {
                        day += days_left as u32;
                        days_left = 0;
                    }
                }
            }

            format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                year, month, day, time_parts.1, time_parts.2, time_parts.3
            )
        }
        Err(_) => String::from("Unknown date"),
    }
}

fn should_skip_entry(
    entry: &fs::DirEntry,
    options: &TreeOptions,
    parent_matched: bool,
) -> std::io::Result<bool> {
    let path = entry.path();
    let file_name = path.file_name().and_then(|name| name.to_str());

    let is_hidden = file_name.map(|name| name.starts_with('.')).unwrap_or(false);
    if !options.all_files && is_hidden {
        return Ok(true);
    }

    for exclude_pattern in &options.exclude_patterns {
        if file_name.is_some_and(|name| exclude_pattern.matches(name)) {
            return Ok(true);
        }
    }

    if let Some(pattern) = &options.pattern_glob {
        if !path.is_dir() && !parent_matched && !file_name.is_some_and(|name| pattern.matches(name))
        {
            return Ok(true);
        }
    }

    if options.dir_only && !path.is_dir() {
        return Ok(true);
    }

    Ok(false)
}

// Helper struct to hold entry and its modification time
struct EntryInfo {
    entry: fs::DirEntry,
    mod_time: io::Result<SystemTime>,
}

// Helper function to format a single line of the tree output
fn format_entry_line(
    entry: &fs::DirEntry,
    options: &TreeOptions,
    indent_state: &[bool],
    is_last: bool,
    icon_manager: &IconManager,
) -> std::io::Result<String> {
    let path = entry.path();
    let mut line = String::new();
    let metadata = entry.metadata()?; // Get metadata early, needed for permissions and potentially size/date/type
    let file_type = metadata.file_type();

    // --- Permissions (optional, Unix only) ---
    if options.print_permissions {
        #[cfg(unix)]
        {
            let mode = metadata.permissions().mode();
            let perms_str = format_permissions_unix(mode, file_type.is_dir());
            line.push_str(&perms_str);
            line.push(' '); // Add space after permissions
        }
        #[cfg(not(unix))] // On non-unix, add placeholder space? Or just nothing?
        {
            // Currently adds nothing on non-Unix platforms
        }
    }

    // --- Indentation ---
    // Indent only if not disabled and depth > 0.
    // Note: indent_state is empty when depth == 0.
    if !options.no_indent && !indent_state.is_empty() {
        for &is_parent_last in indent_state.iter() {
            if is_parent_last {
                line.push_str("    ");
            } else {
                let vertical_line = if options.ascii { "|   " } else { "│   " };
                line.push_str(vertical_line);
            }
        }
    }

    // --- Line Prefix (├──, └──) ---
    let line_prefix = match (options.no_indent, is_last, options.ascii) {
        (true, _, _) => "",
        (false, true, true) => "+---",
        (false, true, false) => "└── ",
        (false, false, true) => "\\---",
        (false, false, false) => "├── ",
    };
    line.push_str(line_prefix);

    // --- Name (potentially colored) ---
    let name_part = if options.full_path {
        path.display().to_string()
    } else {
        entry.file_name().to_string_lossy().to_string()
    };

    // Add icon if enabled
    let display_name = if options.icons {
        let icon = icon_manager.get_icon_for_path(&path);
        format!("{icon} {name_part}")
    } else {
        name_part
    };

    let colored_name = if options.no_color || !options.color {
        display_name
    } else {
        colorize(entry, &display_name)
    };
    line.push_str(&colored_name);

    // --- Append indicator if -F/--classify is enabled ---
    if options.classify {
        let indicator = if file_type.is_dir() {
            "/"
        } else if file_type.is_symlink() {
            "@"
        // Executable check: only on Unix, requires PermissionsExt trait
        } else if file_type.is_file() {
            #[cfg(unix)]
            {
                // Check execute bit for user, group, or others
                if metadata.permissions().mode() & 0o111 != 0 {
                    "*"
                } else {
                    "" // Not executable
                }
            }
            #[cfg(not(unix))] // On non-Unix, files don't get '*' from us
            {
                ""
            }
        } else {
            "" // Default: no indicator for other types (sockets, fifos, etc.)
        };
        line.push_str(indicator);
    }

    // --- Size (optional) ---
    if !file_type.is_dir() && (options.print_size || options.human_readable) {
        let size = metadata.len();
        let size_str = if options.human_readable {
            format!(" [{}]", bytes_to_human_readable(size))
        } else {
            format!(" [{size:5}B]")
        };
        line.push_str(&size_str);
    }

    // --- Modification Date (optional) ---
    if options.print_mod_date {
        match metadata.modified() {
            Ok(mod_time) => {
                let date_str = format!(" [{}]", format_date(mod_time));
                line.push_str(&date_str);
            }
            Err(e) => {
                eprintln!(
                    "Warning: Could not get modification date for {:?}: {}",
                    entry.path(),
                    e
                );
            }
        }
    }

    Ok(line)
}

fn has_pattern_filter(options: &TreeOptions) -> bool {
    options.pattern_glob.is_some() || !options.exclude_patterns.is_empty()
}

fn should_skip_dir_recursion(path: &Path, depth: usize, options: &TreeOptions) -> bool {
    if let Some(max_level) = options.level {
        if (depth + 1) >= max_level as usize {
            return true;
        }
    }
    if let Some(limit) = options.file_limit {
        match fs::read_dir(path) {
            Ok(reader) => {
                if reader.count() > limit as usize {
                    return true;
                }
            }
            Err(e) => {
                eprintln!("Warning: Could not read directory {path:?} to check filelimit: {e}");
            }
        }
    }
    false
}

fn dir_matches_pattern(path: &Path, options: &TreeOptions) -> bool {
    options.match_dirs
        && options.pattern_glob.as_ref().is_some_and(|pattern| {
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|name| pattern.matches(name))
        })
}

#[allow(clippy::too_many_arguments)]
pub fn traverse_directory<P: AsRef<Path>, W: Write>(
    writer: &mut W,
    root_path: P,
    current_path: &Path,
    options: &TreeOptions,
    depth: usize,
    stats: &mut (u64, u64),
    indent_state: &[bool],
    icon_manager: &IconManager,
    parent_matched: bool,
) -> std::io::Result<bool> {
    // --- 1. Read and Pre-process Directory Entries ---
    let read_dir_result = fs::read_dir(current_path);
    let mut entries_info: Vec<EntryInfo> = match read_dir_result {
        Ok(reader) => reader
            .filter_map(Result::ok)
            .filter(
                |entry| match should_skip_entry(entry, options, parent_matched) {
                    Ok(skip) => !skip,
                    Err(e) => {
                        eprintln!(
                            "Warning: Could not apply filters to entry {:?}: {}",
                            entry.path(),
                            e
                        );
                        false
                    }
                },
            )
            .map(|entry| {
                // Get metadata/mod_time needed for sorting
                let mod_time = entry.metadata().and_then(|m| m.modified());
                if let Err(e) = &mod_time {
                    eprintln!(
                        "Warning: Could not get metadata/mod_time for {:?}: {}",
                        entry.path(),
                        e
                    );
                }
                EntryInfo { entry, mod_time }
            })
            .collect(),
        Err(e) => {
            eprintln!("Error reading directory {current_path:?}: {e}");
            return Err(e);
        }
    };

    // --- 2. Sort Entries ---
    if options.dirs_first {
        // Partition into directories and files, handling potential file_type errors
        let (mut dirs, mut files): (Vec<EntryInfo>, Vec<EntryInfo>) =
            std::mem::take(&mut entries_info)
                .into_iter()
                .partition(|info| {
                    // Treat entries where file_type fails as non-directories
                    info.entry
                        .file_type()
                        .map(|ft| ft.is_dir())
                        .unwrap_or(false)
                });

        // Define the comparison logic based on sort_by_time
        let sort_comparison = |a: &EntryInfo, b: &EntryInfo| {
            if options.sort_by_time {
                let time_a = a.mod_time.as_ref().unwrap_or(&SystemTime::UNIX_EPOCH);
                let time_b = b.mod_time.as_ref().unwrap_or(&SystemTime::UNIX_EPOCH);
                time_a
                    .cmp(time_b)
                    .then_with(|| a.entry.file_name().cmp(&b.entry.file_name()))
            } else {
                a.entry.file_name().cmp(&b.entry.file_name())
            }
        };

        // Sort directories and files independently
        dirs.sort_by(sort_comparison);
        files.sort_by(sort_comparison);

        // Apply reverse if needed
        if options.reverse {
            dirs.reverse();
            files.reverse();
        }

        // Combine back, dirs first
        entries_info = dirs;
        entries_info.append(&mut files);
    } else {
        // Original sorting logic if dirs_first is not enabled
        if options.sort_by_time {
            entries_info.sort_by(|a, b| {
                let time_a = a.mod_time.as_ref().unwrap_or(&SystemTime::UNIX_EPOCH);
                let time_b = b.mod_time.as_ref().unwrap_or(&SystemTime::UNIX_EPOCH);
                time_a
                    .cmp(time_b)
                    .then_with(|| a.entry.file_name().cmp(&b.entry.file_name()))
            });
        } else {
            entries_info.sort_by_key(|info| info.entry.file_name().to_owned());
        }
        if options.reverse {
            entries_info.reverse();
        }
    }

    let prune_mode = options.prune && has_pattern_filter(options) && !options.dir_only;
    let mut found_content = false;

    let surviving: Vec<bool> = if prune_mode {
        entries_info
            .iter()
            .map(|info| {
                let path = info.entry.path();
                if info.entry.file_type()?.is_dir() {
                    let skip = should_skip_dir_recursion(&path, depth, options);
                    let child_matched = dir_matches_pattern(&path, options);
                    let has_content = if skip {
                        false
                    } else {
                        let mut probe_stats = (0u64, 0u64);
                        traverse_directory(
                            &mut io::sink(),
                            root_path.as_ref(),
                            &path,
                            options,
                            depth + 1,
                            &mut probe_stats,
                            &[],
                            icon_manager,
                            child_matched,
                        )?
                    };
                    Ok(has_content || child_matched)
                } else {
                    Ok(true)
                }
            })
            .collect::<std::io::Result<_>>()?
    } else {
        vec![true; entries_info.len()]
    };

    let surviving_count = surviving.iter().filter(|&&f| f).count();
    let mut emit_idx = 0;

    for (index, info) in entries_info.into_iter().enumerate() {
        if !surviving[index] {
            continue;
        }
        let entry = info.entry;
        let path = entry.path();
        let is_last = emit_idx == surviving_count.saturating_sub(1);
        emit_idx += 1;

        if entry.file_type()?.is_dir() {
            let skip_recursion = should_skip_dir_recursion(&path, depth, options);
            let child_parent_matched = dir_matches_pattern(&path, options);

            let line = format_entry_line(&entry, options, indent_state, is_last, icon_manager)?;
            writeln!(writer, "{line}")?;
            stats.0 += 1;
            found_content = true;

            if !skip_recursion {
                let mut next_indent_state = indent_state.to_vec();
                next_indent_state.push(is_last);
                traverse_directory(
                    writer,
                    root_path.as_ref(),
                    &path,
                    options,
                    depth + 1,
                    stats,
                    &next_indent_state,
                    icon_manager,
                    child_parent_matched,
                )?;
            }
        } else {
            let line = format_entry_line(&entry, options, indent_state, is_last, icon_manager)?;
            writeln!(writer, "{line}")?;
            stats.1 += 1;
            found_content = true;
        }
    }

    Ok(found_content)
}

pub fn list_directory<P: AsRef<Path>>(path: P, options: &TreeOptions) -> std::io::Result<()> {
    if options.from_file {
        list_from_input(path.as_ref(), options)
    } else {
        list_from_filesystem(path.as_ref(), options)
    }
}

/// Lists the directory structure and returns it as a String instead of writing to stdout.
///
/// # Arguments
/// * `path` - The path to list
/// * `options` - TreeOptions to control the output format
///
/// # Returns
/// * `Ok(String)` - The formatted directory tree as a string
/// * `Err(io::Error)` - If an error occurs during traversal
///
/// # Example
/// ```rust,no_run
/// use rust_tree::rust_tree::options::TreeOptions;
/// use rust_tree::rust_tree::traversal::list_directory_as_string;
///
/// let options = TreeOptions {
///     all_files: false,
///     level: None,
///     full_path: false,
///     dir_only: false,
///     no_indent: false,
///     print_size: false,
///     human_readable: false,
///     pattern_glob: None,
///     match_dirs: false,
///     exclude_patterns: vec![],
///     color: false,
///     no_color: false,
///     ascii: false,
///     sort_by_time: false,
///     reverse: false,
///     print_mod_date: false,
///     output_file: None,
///     file_limit: None,
///     dirs_first: false,
///     classify: false,
///     no_report: false,
///     print_permissions: false,
///     from_file: false,
///     icons: false,
///     prune: false,
///     gitignore: false,
/// };
/// let tree_output = list_directory_as_string(".", &options).unwrap();
/// println!("{}", tree_output);
/// ```
pub fn list_directory_as_string<P: AsRef<Path>>(
    path: P,
    options: &TreeOptions,
) -> std::io::Result<String> {
    let mut buffer = Vec::new();

    if options.from_file {
        list_from_input_with_writer(path.as_ref(), options, &mut buffer)?;
    } else {
        list_from_filesystem_with_writer(path.as_ref(), options, &mut buffer)?;
    }

    String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn create_writer(options: &TreeOptions) -> std::io::Result<Box<dyn Write>> {
    match &options.output_file {
        Some(filename) => {
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(filename)?;
            Ok(Box::new(BufWriter::new(file)))
        }
        None => Ok(Box::new(BufWriter::new(io::stdout()))),
    }
}

fn list_from_input(input_path: &Path, options: &TreeOptions) -> std::io::Result<()> {
    let writer = create_writer(options)?;
    list_from_input_with_writer(input_path, options, writer)
}

fn list_from_input_with_writer<W: Write>(
    input_path: &Path,
    options: &TreeOptions,
    writer: W,
) -> std::io::Result<()> {
    let lines = read_file_listing(input_path)?;
    let entries = parse_file_listing(lines);
    let virtual_tree = build_virtual_tree(entries, options);
    display_virtual_tree_with_writer(virtual_tree, options, writer)
}

fn list_from_filesystem(current_path: &Path, options: &TreeOptions) -> std::io::Result<()> {
    let writer = create_writer(options)?;
    list_from_filesystem_with_writer(current_path, options, writer)
}

fn list_from_filesystem_with_writer<W: Write>(
    current_path: &Path,
    options: &TreeOptions,
    mut writer: W,
) -> std::io::Result<()> {
    let display_path = if options.full_path {
        current_path.canonicalize()?.display().to_string()
    } else {
        current_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(".")
            .to_string()
    };
    writeln!(writer, "{display_path}")?;

    let mut stats = (0, 0); // (directories, files)

    // Create icon manager if icons are enabled
    let icon_manager = IconManager::new();

    traverse_directory(
        &mut writer,
        current_path,
        current_path,
        options,
        0,
        &mut stats,
        &[],
        &icon_manager,
        false,
    )?;

    if !options.no_report {
        let dir_str = if stats.0 == 1 {
            "directory"
        } else {
            "directories"
        };
        if options.dir_only {
            writeln!(writer, "\n{} {}", stats.0, dir_str)?;
        } else {
            let file_str = if stats.1 == 1 { "file" } else { "files" };
            writeln!(
                writer,
                "\n{} {}, {} {}",
                stats.0, dir_str, stats.1, file_str
            )?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn display_virtual_tree_with_writer<W: Write>(
    virtual_tree: VirtualTree,
    options: &TreeOptions,
    mut writer: W,
) -> std::io::Result<()> {
    use std::collections::HashMap;

    writeln!(writer, "{}", virtual_tree.root_name)?;

    // Build hierarchy
    let mut children: HashMap<String, Vec<&FileEntry>> = HashMap::new();
    let mut all_entries: HashMap<String, &FileEntry> = HashMap::new();

    for entry in &virtual_tree.entries {
        all_entries.insert(entry.path.clone(), entry);

        if entry.path.is_empty() {
            continue;
        }

        let parent = if let Some(pos) = entry.path.rfind('/') {
            entry.path[..pos].to_string()
        } else {
            String::new()
        };

        children.entry(parent).or_default().push(entry);
    }

    // Sort entries in each directory
    for child_list in children.values_mut() {
        child_list.sort_by(|a, b| {
            if options.dirs_first {
                match (a.is_dir, b.is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.path.cmp(&b.path),
                }
            } else {
                a.path.cmp(&b.path)
            }
        });
    }

    let mut stats = (0, 0); // (directories, files)

    // Create icon manager
    let icon_manager = IconManager::new();

    // Display tree starting from root
    if let Some(root_children) = children.get("") {
        display_virtual_entries(
            &mut writer,
            root_children,
            &children,
            options,
            &mut stats,
            &[],
            0,
            &icon_manager,
            false,
        )?;
    }

    if !options.no_report {
        let dir_str = if stats.0 == 1 {
            "directory"
        } else {
            "directories"
        };
        if options.dir_only {
            writeln!(writer, "\n{} {}", stats.0, dir_str)?;
        } else {
            let file_str = if stats.1 == 1 { "file" } else { "files" };
            writeln!(
                writer,
                "\n{} {}, {} {}",
                stats.0, dir_str, stats.1, file_str
            )?;
        }
    }

    writer.flush()?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn display_virtual_entries<W: Write>(
    writer: &mut W,
    entries: &[&FileEntry],
    all_children: &std::collections::HashMap<String, Vec<&FileEntry>>,
    options: &TreeOptions,
    stats: &mut (u32, u32),
    indent_state: &[bool],
    depth: usize,
    icon_manager: &IconManager,
    parent_matched: bool,
) -> std::io::Result<bool> {
    if let Some(max_level) = options.level {
        if depth >= max_level as usize {
            return Ok(false);
        }
    }

    let prune_mode = options.prune && has_pattern_filter(options) && !options.dir_only;
    let mut found_content = false;

    let surviving: Vec<bool> = entries
        .iter()
        .map(|entry| {
            if entry.is_dir && prune_mode {
                if should_skip_virtual_entry(entry, options, parent_matched)? {
                    return Ok(false);
                }
                let filename = if let Some(pos) = entry.path.rfind('/') {
                    &entry.path[pos + 1..]
                } else {
                    &entry.path
                };
                let child_matched = options.match_dirs
                    && options
                        .pattern_glob
                        .as_ref()
                        .is_some_and(|pattern| pattern.matches(filename));
                let has_content = if let Some(children) = all_children.get(&entry.path) {
                    let mut probe_stats = (0u32, 0u32);
                    display_virtual_entries(
                        &mut io::sink(),
                        children,
                        all_children,
                        options,
                        &mut probe_stats,
                        &[],
                        depth + 1,
                        icon_manager,
                        child_matched,
                    )?
                } else {
                    false
                };
                Ok(has_content || child_matched)
            } else {
                let should_skip = should_skip_virtual_entry(entry, options, parent_matched)?;
                Ok(!should_skip)
            }
        })
        .collect::<std::io::Result<_>>()?;

    let surviving_count = surviving.iter().filter(|&&f| f).count();
    let mut emit_idx = 0;

    for (i, entry) in entries.iter().enumerate() {
        if !surviving[i] {
            continue;
        }
        let is_last = emit_idx == surviving_count.saturating_sub(1);
        emit_idx += 1;

        if entry.is_dir {
            let filename = if let Some(pos) = entry.path.rfind('/') {
                &entry.path[pos + 1..]
            } else {
                &entry.path
            };
            let child_parent_matched = options.match_dirs
                && options
                    .pattern_glob
                    .as_ref()
                    .is_some_and(|pattern| pattern.matches(filename));

            stats.0 += 1;
            display_virtual_entry(writer, entry, options, indent_state, is_last, icon_manager)?;
            found_content = true;

            if let Some(children) = all_children.get(&entry.path) {
                let mut new_indent_state = indent_state.to_vec();
                new_indent_state.push(!is_last);
                display_virtual_entries(
                    writer,
                    children,
                    all_children,
                    options,
                    stats,
                    &new_indent_state,
                    depth + 1,
                    icon_manager,
                    child_parent_matched,
                )?;
            }
        } else {
            stats.1 += 1;
            display_virtual_entry(writer, entry, options, indent_state, is_last, icon_manager)?;
            found_content = true;
        }
    }

    Ok(found_content)
}

fn display_virtual_entry<W: Write>(
    writer: &mut W,
    entry: &FileEntry,
    options: &TreeOptions,
    indent_state: &[bool],
    is_last: bool,
    icon_manager: &IconManager,
) -> std::io::Result<()> {
    // Build prefix
    let mut prefix = String::new();
    if !options.no_indent {
        for &has_sibling in indent_state {
            if options.ascii {
                prefix.push_str(if has_sibling { "|   " } else { "    " });
            } else {
                prefix.push_str(if has_sibling { "│   " } else { "    " });
            }
        }

        if options.ascii {
            prefix.push_str(if is_last { "`-- " } else { "|-- " });
        } else {
            prefix.push_str(if is_last { "└── " } else { "├── " });
        }
    }

    // Get filename
    let filename = if let Some(pos) = entry.path.rfind('/') {
        &entry.path[pos + 1..]
    } else {
        &entry.path
    };

    // Build display name
    let mut display_name = if options.full_path {
        entry.path.clone()
    } else {
        filename.to_string()
    };

    // Add icon if enabled
    if options.icons {
        let path = Path::new(&entry.path);
        let icon = icon_manager.get_icon_for_path(path);
        display_name = format!("{icon} {display_name}");
    }

    // Add file type indicator
    if options.classify && entry.is_dir {
        display_name.push('/');
    }

    // Add size if requested
    if options.print_size || options.human_readable {
        if let Some(size) = entry.size {
            let size_str = if options.human_readable {
                bytes_to_human_readable(size)
            } else {
                size.to_string()
            };
            display_name = format!("[{size_str}]  {display_name}");
        }
    }

    // Apply colorization (simplified for virtual entries)
    let colored_name = display_name;

    writeln!(writer, "{prefix}{colored_name}")?;
    Ok(())
}

fn should_skip_virtual_entry(
    entry: &FileEntry,
    options: &TreeOptions,
    parent_matched: bool,
) -> std::io::Result<bool> {
    let filename = if let Some(pos) = entry.path.rfind('/') {
        &entry.path[pos + 1..]
    } else {
        &entry.path
    };

    let is_hidden = filename.starts_with('.');
    if !options.all_files && is_hidden {
        return Ok(true);
    }

    for exclude_pattern in &options.exclude_patterns {
        if exclude_pattern.matches(filename) {
            return Ok(true);
        }
    }

    if let Some(include_pattern) = &options.pattern_glob {
        if !entry.is_dir && !parent_matched && !include_pattern.matches(filename) {
            return Ok(true);
        }
    }

    if options.dir_only && !entry.is_dir {
        return Ok(true);
    }

    Ok(false)
}
