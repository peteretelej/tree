use std::fs;
use std::io;
use std::path::Path;
use std::time::SystemTime;

use crate::rust_tree::display::colorize;
use crate::rust_tree::options::TreeOptions;
use crate::rust_tree::utils::bytes_to_human_readable;

fn should_skip_entry(
    entry: &fs::DirEntry,
    options: &TreeOptions,
    depth: usize,
) -> std::io::Result<bool> {
    let path = entry.path();
    let file_name = path.file_name().and_then(|name| name.to_str());

    // Check depth limit first (most common)
    if let Some(max_level) = options.level {
        if depth >= max_level as usize { // depth is 0-indexed, level is 1-indexed
            return Ok(true); // Skip if depth is at or beyond the limit
        }
    }

    // Check if hidden
    let is_hidden = file_name.map(|name| name.starts_with('.')).unwrap_or(false);
    if !options.all_files && is_hidden {
        return Ok(true);
    }

    // Check exclude pattern FIRST
    if let Some(exclude_pattern) = &options.exclude_pattern {
        if file_name.is_some_and(|name| exclude_pattern.matches(name)) {
            return Ok(true); // Skip if matches exclude pattern
        }
    }

    // Check include pattern (only if exclude didn't match)
    if let Some(pattern) = &options.pattern_glob {
        // Directories are not filtered by pattern, only files
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

    Ok(line)
}

pub fn traverse_directory<P: AsRef<Path>>(
    root_path: P,
    current_path: &Path,
    options: &TreeOptions,
    depth: usize,
    stats: &mut (u64, u64),
    indent_state: &[bool],
) -> std::io::Result<()> {
    // Attempt to read directory entries, filter out errors, then collect.
    let read_dir_result = fs::read_dir(current_path);
    let mut entries_info: Vec<EntryInfo> = match read_dir_result {
        Ok(reader) => reader
            .filter_map(Result::ok) // Ignore entries that cause an error during iteration
            .map(|entry| {
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
            // If reading the directory itself fails (e.g., permission denied), print error and skip.
            eprintln!("Error reading directory {:?}: {}", current_path, e);
            return Ok(()); // Continue traversal for other directories if possible
        }
    };

    // Sort entries based on options
    if options.sort_by_time {
        entries_info.sort_by(|a, b| {
            // Treat errors during mod_time retrieval by sorting them first (using UNIX_EPOCH)
            let time_a = a.mod_time.as_ref().unwrap_or(&SystemTime::UNIX_EPOCH);
            let time_b = b.mod_time.as_ref().unwrap_or(&SystemTime::UNIX_EPOCH);
            time_a
                .cmp(time_b)
                .then_with(|| a.entry.file_name().cmp(&b.entry.file_name())) // Secondary sort by name
        });
    } else {
        // Default sort by filename
        entries_info.sort_by_key(|info| info.entry.file_name().to_owned());
    }

    let last_index = entries_info.len().saturating_sub(1);

    for (index, info) in entries_info.into_iter().enumerate() {
        let entry = info.entry; // Extract the DirEntry from EntryInfo
        let path = entry.path();
        let is_entry_last = index == last_index;

        // Check if entry should be skipped (using the extracted 'entry')
        if should_skip_entry(&entry, options, depth)? {
            continue;
        }

        // Format the line for the current entry
        let line = format_entry_line(&entry, options, indent_state, is_entry_last)?;
        println!("{}", line);

        // Recurse into directories if necessary
        if entry.file_type()?.is_dir() {
            stats.0 += 1;
            // Recurse ONLY if the NEXT level is within the limit
            let should_recurse = options.level.is_none()
                || (depth + 1) < options.level.unwrap() as usize;

            if should_recurse {
                // Prepare the indent state for the recursive call
                let mut next_indent_state = indent_state.to_vec();
                next_indent_state.push(is_entry_last);
                
                traverse_directory(
                    root_path.as_ref(),
                    &path,
                    options,
                    depth + 1,
                    stats,
                    &next_indent_state,
                )?;
            }
        } else {
            stats.1 += 1;
            // No newline print needed here anymore, handled by format_entry_line + println
        }
    }

    Ok(())
}

pub fn list_directory<P: AsRef<Path>>(path: P, options: &TreeOptions) -> std::io::Result<()> {
    let current_path = path.as_ref();
    let display_path = if options.full_path {
        current_path.canonicalize()?.display().to_string()
    } else {
        current_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(".")
            .to_string()
    };
    println!("{}", display_path);

    let mut stats = (0, 0); // (directories, files)

    // Start the recursive traversal with empty initial indent state
    traverse_directory(
        current_path, // Use original path for root comparison
        current_path,
        options,
        0, // Initial depth for root's contents
        &mut stats,
        &[], // Initial empty indent state
    )?;

    println!("\n{} directories, {} files", stats.0, stats.1);
    Ok(())
}
