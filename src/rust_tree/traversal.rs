use std::fs;
use std::path::Path;
use std::time::SystemTime;
use std::fs::{OpenOptions};
use std::io::{BufWriter, Write};
use std::io;

use crate::rust_tree::display::colorize;
use crate::rust_tree::options::TreeOptions;
use crate::rust_tree::utils::bytes_to_human_readable;
use chrono::{DateTime, Local};

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

    // --- Size (optional) ---
    let file_type = entry.file_type()?;
    if !file_type.is_dir() && (options.print_size || options.human_readable) {
        let metadata = entry.metadata()?;
        let size = metadata.len();
        let size_str = if options.human_readable {
            format!(" [{}]", bytes_to_human_readable(size))
        } else {
            format!(" [{:5}B]", size)
        };
        line.push_str(&size_str);
    }

    // --- Modification Date (optional) ---
    if options.print_mod_date {
        let metadata = entry.metadata()?;
        match metadata.modified() {
            Ok(mod_time) => {
                let datetime: DateTime<Local> = mod_time.into();
                // Format like YYYY-MM-DD HH:MM:SS
                let date_str = format!(" [{}]", datetime.format("%Y-%m-%d %H:%M:%S"));
                line.push_str(&date_str);
            }
            Err(e) => {
                eprintln!("Warning: Could not get modification date for {:?}: {}", entry.path(), e);
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
            eprintln!("Error reading directory {:?}: {}", current_path, e);
            return Ok(());
        }
    };

    // --- 2. Sort Entries --- 
    if options.sort_by_time { // Sort logic remains the same
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

    // --- 4. Iterate, Print, Count Stats, and Recurse (conditionally) --- 
    let last_index = entries_info.len().saturating_sub(1);
    for (index, info) in entries_info.into_iter().enumerate() {
        let entry = info.entry;
        let path = entry.path();
        let is_entry_last = index == last_index;

        // Format and print the line for the current entry
        let line = format_entry_line(&entry, options, indent_state, is_entry_last)?;
        writeln!(writer, "{}", line)?;

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
                    match fs::read_dir(&path) { // Read the *child* directory (`path`) to count its entries
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
                                "Warning: Could not read directory {:?} to check filelimit: {}",
                                path,
                                e
                            );
                        }
                    }
                }
            }
            // --- End limit checks --- 

            if !skip_recursion { // Recurse only if no limits apply
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
        } else { // It's a file
            stats.1 += 1; // Count this file
        }
    }

    Ok(())
}

pub fn list_directory<P: AsRef<Path>>(path: P, options: &TreeOptions) -> std::io::Result<()> {
    let current_path = path.as_ref();

    // Determine output writer: file or stdout
    let mut writer: Box<dyn Write> = match &options.output_file {
        Some(filename) => {
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true) // Overwrite existing file
                .open(filename)?;
            Box::new(BufWriter::new(file))
        }
        None => Box::new(BufWriter::new(io::stdout())),
    };

    let display_path = if options.full_path {
        current_path.canonicalize()?.display().to_string()
    } else {
        current_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(".")
            .to_string()
    };
    //println!("{}", display_path);
    writeln!(writer, "{}", display_path)?; // Use the writer

    let mut stats = (0, 0); // (directories, files)

    // Start the recursive traversal with empty initial indent state
    traverse_directory(
        &mut writer, // Pass the writer
        current_path, // Use original path for root comparison
        current_path,
        options,
        0, // Initial depth for root's contents
        &mut stats,
        &[], // Initial empty indent state
    )?;

    //println!("\n{} directories, {} files", stats.0, stats.1);
    let dir_str = if stats.0 == 1 { "directory" } else { "directories" };
    let file_str = if stats.1 == 1 { "file" } else { "files" };
    writeln!(writer, "\n{} {}, {} {}", stats.0, dir_str, stats.1, file_str)?;
    writer.flush()?; // Explicitly flush the buffer
    Ok(())
}
