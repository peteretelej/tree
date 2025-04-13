use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::rust_tree::display::colorize;
use crate::rust_tree::options::TreeOptions;
use crate::rust_tree::utils::bytes_to_human_readable;

fn normalize_pattern(pattern: &str) -> String {
    // Convert Windows path separators to Unix style for consistent matching
    pattern.replace('\\', "/").trim().to_string()
}

fn matches_any_pattern(name: &str, pattern: &glob::Pattern) -> bool {
    let normalized_name = normalize_pattern(name);
    pattern.as_str()
        .split('|')
        .any(|p| {
            let normalized_pattern = normalize_pattern(p);
            glob::Pattern::new(&normalized_pattern)
                .map(|single_pattern| single_pattern.matches(&normalized_name))
                .unwrap_or(false)
        })
}

pub fn traverse_directory<P: AsRef<Path>>(
    root_path: P,
    current_path: &Path,
    options: &TreeOptions,
    depth: usize,
    _is_last: bool,
    stats: &mut (u64, u64),
    last_entry_depths: &mut HashSet<usize>,
) -> std::io::Result<()> {
    let mut entries: Vec<_> = fs::read_dir(current_path)?.collect();
    entries.sort_by_key(|entry| entry.as_ref().unwrap().file_name().to_owned());

    let last_index = entries.len().saturating_sub(1);

    for (index, entry) in entries.into_iter().enumerate() {
        let entry = entry?;
        let path = entry.path();
        let is_entry_last = index == last_index;

        let is_hidden = path
            .file_name()
            .map(|name| name.to_string_lossy().starts_with('.'))
            .unwrap_or(false);

        let should_skip = match (options.all_files, is_hidden, options.level) {
            (false, true, _) => true,
            (_, _, Some(max_depth)) if depth >= max_depth as usize => true,
            _ => false
        };
        if should_skip {
            continue;
        }

        let file_name = path.file_name().unwrap().to_string_lossy();
        if let Some(exclude_pattern) = &options.exclude_pattern {
            if matches_any_pattern(&file_name, exclude_pattern) {
                continue;
            }
        }

        let is_dir = path.is_dir();
        let should_include = match (&options.pattern_glob, is_dir) {
            (Some(pattern), false) => matches_any_pattern(&file_name, pattern),
            _ => true,
        };
        if !should_include {
            continue;
        }

        if options.dir_only && !is_dir {
            continue;
        }

        let root_path_buf = root_path.as_ref().to_path_buf();
        let current_path_buf = current_path.to_path_buf();
        if !options.no_indent && current_path_buf != root_path_buf {
            for depth_level in 0..depth {
                let indent = if last_entry_depths.contains(&depth_level) {
                    "    "
                } else {
                    if options.ascii { "|   " } else { "│   " }
                };
                print!("{}", indent);
            }
        }

        let prefix = match (options.no_indent, is_entry_last, options.ascii) {
            (true, _, _) => "",
            (false, true, true) => "+---",
            (false, true, false) => "└── ",
            (false, false, true) => "\\---",
            (false, false, false) => "├── ",
        };

        let display_name = if options.full_path {
            // Use platform-specific display for full paths
            path.display().to_string()
        } else {
            entry.file_name().to_string_lossy().to_string()
        };
        let formatted_name = if options.no_color || !options.color {
            display_name
        } else {
            colorize(&entry, display_name)
        };
        print!("{}{}", prefix, formatted_name);

        if is_dir {
            if !is_hidden {
                stats.0 += 1;
            }
            println!();
            if is_entry_last {
                last_entry_depths.insert(depth);
            }
            traverse_directory(
                root_path.as_ref(),
                &path,
                options,
                depth + 1,
                is_entry_last,
                stats,
                last_entry_depths,
            )?;
            if is_entry_last {
                last_entry_depths.remove(&depth);
            }
        } else {
            if !is_hidden {
                stats.1 += 1;
            }
            if options.print_size || options.human_readable {
                let size = entry.metadata()?.len();
                let size_display = if options.human_readable {
                    format!(" ({})", bytes_to_human_readable(size))
                } else {
                    format!(" ({:5}B)", size)
                };
                print!("{}", size_display);
            }
            println!();
        }
    }

    Ok(())
}

pub fn list_directory<P: AsRef<Path>>(path: P, options: &TreeOptions) -> std::io::Result<()> {
    let current_path = path.as_ref();
    println!(
        "{}",
        current_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(".")
    );

    let mut stats = (0, 0); // (directories, files)
                            // Recursively traverse the directory and print its contents
    let mut last_entry_depths = HashSet::new();

    traverse_directory(
        current_path,
        current_path,
        options,
        0,
        false,
        &mut stats,
        &mut last_entry_depths,
    )?;

    println!("\n{} directories, {} files", stats.0, stats.1);
    Ok(())
}
