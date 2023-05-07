use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::rust_tree::options::TreeOptions;
use crate::rust_tree::utils::bytes_to_human_readable;

pub fn traverse_directory<P: AsRef<Path>>(
    root_path: P,
    current_path: &Path,
    options: &TreeOptions,
    depth: usize,
    is_last: bool,
    stats: &mut (u64, u64),
    last_entry_depths: &mut HashSet<usize>,
) -> std::io::Result<()> {
    let entries: Vec<_> = fs::read_dir(current_path)?.collect();
    let last_index = entries.len().saturating_sub(1);

    for (index, entry) in entries.into_iter().enumerate() {
        let entry = entry?;
        let path = entry.path();
        let is_entry_last = index == last_index;

        // Check if hidden files and directories are allowed
        let is_hidden = path
            .file_name()
            .map(|name| name.to_string_lossy().starts_with('.'))
            .unwrap_or(false);
        if !options.all_files && is_hidden {
            continue;
        }
        if options.level.is_some() && depth >= options.level.unwrap() as usize {
            continue;
        }
        if options.pattern_glob.is_some() && !path.is_dir() {
            let pattern_glob = options.pattern_glob.as_ref().unwrap();
            let file_name = path.file_name().unwrap().to_string_lossy();
            if !pattern_glob.matches(&file_name) {
                continue;
            }
        }
        if options.dir_only && !path.is_dir() {
            continue;
        }

        // Print indentation
        let root_path_buf = root_path.as_ref().to_path_buf();
        let current_path_buf = current_path.to_path_buf();
        if !options.no_indent && current_path_buf != root_path_buf {
            for i in 0..depth {
                if last_entry_depths.contains(&i) {
                    print!("    ");
                } else {
                    print!("│   ");
                }
            }
            if is_last {
                print!("    ");
            }
        }

        // Print file/directory name with prefix
        let prefix = if options.no_indent {
            ""
        } else if is_entry_last {
            "└── "
        } else {
            "├── "
        };

        let name = if options.full_path {
            path.display().to_string()
        } else {
            entry.file_name().to_string_lossy().to_string()
        };
        print!("{}{}", prefix, name);

        if entry.file_type()?.is_dir() {
            // If it's a directory, recurse into it
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
            // If it's a file and the size option is set, print its size
            if !is_hidden {
                stats.1 += 1;
            }
            if options.print_size || options.human_readable {
                let metadata = entry.metadata()?;
                let size = metadata.len();
                let size_str = if options.human_readable {
                    format!(" ({})", bytes_to_human_readable(size))
                } else {
                    format!(" ({:5}B)", size)
                };
                print!("{}", size_str);
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
