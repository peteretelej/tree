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
    _depth: usize,
) -> std::io::Result<bool> {
    let path = entry.path();
    let file_name = path.file_name().and_then(|name| name.to_str());

    // Check if hidden
    let is_hidden = file_name.map(|name| name.starts_with('.')).unwrap_or(false);
    if !options.all_files && is_hidden {
        return Ok(true);
    }

    // Check exclude pattern FIRST
    if let Some(exclude_pattern) = &options.exclude_pattern {
        if file_name.is_some_and(|name| exclude_pattern.matches(name)) {
            return Ok(true);
        }
    }

    // Check include pattern (only if exclude didn't match)
    if let Some(pattern) = &options.pattern_glob {
        if !path.is_dir() && !file_name.is_some_and(|name| pattern.matches(name)) {
            return Ok(true);
        }
    }

    // Check dir_only
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
    let colored_name = if options.no_color || !options.color {
        name_part
    } else {
        colorize(entry, &name_part)
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

pub fn traverse_directory<P: AsRef<Path>, W: Write>(
    writer: &mut W,
    root_path: P,
    current_path: &Path,
    options: &TreeOptions,
    depth: usize,
    stats: &mut (u64, u64),
    indent_state: &[bool],
) -> std::io::Result<()> {
    // --- 1. Read and Pre-process Directory Entries ---
    let read_dir_result = fs::read_dir(current_path);
    let mut entries_info: Vec<EntryInfo> = match read_dir_result {
        Ok(reader) => reader
            .filter_map(Result::ok) // Ignore entries that cause an error during iteration
            .filter(|entry| {
                // Pre-filter based on options *not* related to recursion limits (level/filelimit)
                // This ensures stats are counted correctly even if recursion is skipped.
                match should_skip_entry(entry, options, depth) {
                    Ok(skip) => !skip,
                    Err(e) => {
                        eprintln!(
                            "Warning: Could not apply filters to entry {:?}: {}",
                            entry.path(),
                            e
                        );
                        false // Skip if error occurs during filtering
                    }
                }
            })
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

    // --- 4. Iterate, Print, Count Stats, and Recurse (conditionally) ---
    let last_index = entries_info.len().saturating_sub(1);
    for (index, info) in entries_info.into_iter().enumerate() {
        let entry = info.entry;
        let path = entry.path();
        let is_entry_last = index == last_index;

        // Format and print the line for the current entry
        let line = format_entry_line(&entry, options, indent_state, is_entry_last)?;
        writeln!(writer, "{line}")?;

        // Update stats and decide recursion based on entry type
        if entry.file_type()?.is_dir() {
            stats.0 += 1; // Count this directory

            // --- Check limits specifically for this directory before recursion ---
            let mut skip_recursion = false;

            // 1. Check level limit for the *next* level
            if let Some(max_level) = options.level {
                // If the next depth (depth + 1) is >= max_level, we should not recurse.
                // Remember depth is 0-indexed, max_level is 1-indexed.
                if (depth + 1) >= max_level as usize {
                    skip_recursion = true;
                }
            }

            // 2. Check file limit (only if level limit doesn't already skip)
            if !skip_recursion {
                if let Some(limit) = options.file_limit {
                    match fs::read_dir(&path) {
                        // Read the *child* directory (`path`) to count its entries
                        Ok(reader) => {
                            // Use iterator `count()` for efficiency.
                            // This counts *raw* entries, matching standard `tree --filelimit` behavior.
                            let entry_count = reader.count();
                            if entry_count > limit as usize {
                                skip_recursion = true;
                                // Optionally: Add indicator like "[...]" to the printed line if skipped?
                            }
                        }
                        Err(e) => {
                            // If we can't read the directory to check the limit, log warning but don't skip.
                            eprintln!(
                                "Warning: Could not read directory {path:?} to check filelimit: {e}"
                            );
                        }
                    }
                }
            }
            // --- End limit checks ---

            if !skip_recursion {
                // Recurse only if no limits apply
                let mut next_indent_state = indent_state.to_vec();
                next_indent_state.push(is_entry_last);
                traverse_directory(
                    writer,
                    root_path.as_ref(),
                    &path, // Recurse into the child directory `path`
                    options,
                    depth + 1,
                    stats,
                    &next_indent_state,
                )?;
            }
        } else {
            // It's a file
            stats.1 += 1; // Count this file
        }
    }

    Ok(())
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
///     exclude_pattern: None,
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

    // Start the recursive traversal with empty initial indent state
    traverse_directory(
        &mut writer,  // Pass the writer
        current_path, // Use original path for root comparison
        current_path,
        options,
        0, // Initial depth for root's contents
        &mut stats,
        &[], // Initial empty indent state
    )?;

    // Print summary only if --noreport is not set
    if !options.no_report {
        //println!("\n{} directories, {} files", stats.0, stats.1);
        let dir_str = if stats.0 == 1 {
            "directory"
        } else {
            "directories"
        };
        let file_str = if stats.1 == 1 { "file" } else { "files" };
        writeln!(
            writer,
            "\n{} {}, {} {}",
            stats.0, dir_str, stats.1, file_str
        )?;
    }

    writer.flush()?; // Explicitly flush the buffer
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
        )?;
    }

    // Print summary only if --noreport is not set
    if !options.no_report {
        let dir_str = if stats.0 == 1 {
            "directory"
        } else {
            "directories"
        };
        let file_str = if stats.1 == 1 { "file" } else { "files" };
        writeln!(
            writer,
            "\n{} {}, {} {}",
            stats.0, dir_str, stats.1, file_str
        )?;
    }

    writer.flush()?;
    Ok(())
}

fn display_virtual_entries<W: Write>(
    writer: &mut W,
    entries: &[&FileEntry],
    all_children: &std::collections::HashMap<String, Vec<&FileEntry>>,
    options: &TreeOptions,
    stats: &mut (u32, u32),
    indent_state: &[bool],
    depth: usize,
) -> std::io::Result<()> {
    // Check level limit
    if let Some(max_level) = options.level {
        if depth >= max_level as usize {
            return Ok(());
        }
    }

    for (i, entry) in entries.iter().enumerate() {
        let is_last = i == entries.len() - 1;

        // Apply filters
        if should_skip_virtual_entry(entry, options)? {
            continue;
        }

        // Update stats
        if entry.is_dir {
            stats.0 += 1;
        } else {
            stats.1 += 1;
        }

        // Display entry
        display_virtual_entry(writer, entry, options, indent_state, is_last)?;

        // Recurse into directories
        if entry.is_dir {
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
                )?;
            }
        }
    }

    Ok(())
}

fn display_virtual_entry<W: Write>(
    writer: &mut W,
    entry: &FileEntry,
    options: &TreeOptions,
    indent_state: &[bool],
    is_last: bool,
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

fn should_skip_virtual_entry(entry: &FileEntry, options: &TreeOptions) -> std::io::Result<bool> {
    let filename = if let Some(pos) = entry.path.rfind('/') {
        &entry.path[pos + 1..]
    } else {
        &entry.path
    };

    // Check if hidden
    let is_hidden = filename.starts_with('.');
    if !options.all_files && is_hidden {
        return Ok(true);
    }

    // Check exclude pattern
    if let Some(exclude_pattern) = &options.exclude_pattern {
        if exclude_pattern.matches(filename) {
            return Ok(true);
        }
    }

    // Check include pattern
    if let Some(include_pattern) = &options.pattern_glob {
        if !include_pattern.matches(filename) {
            return Ok(true);
        }
    }

    // Check directories only
    if options.dir_only && !entry.is_dir {
        return Ok(true);
    }

    Ok(false)
}
