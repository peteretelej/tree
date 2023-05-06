use std::fs;
use std::path::Path;

use crate::tree::TreeOptions;

pub fn traverse_directory<P: AsRef<Path>>(
    root_path: P,
    current_path: &Path,
    options: &TreeOptions,
    depth: usize,
    is_last: bool,
    stats: &mut (u64, u64),
) -> std::io::Result<()> {
    let entries: Vec<_> = fs::read_dir(current_path)?.collect();
    let last_index = entries.len().saturating_sub(1);

    for (index, entry) in entries.into_iter().enumerate() {
        let entry = entry?;
        let path = entry.path();
        let is_entry_last = index == last_index;

        // Check if hidden files and directories are allowed
        let is_hidden = path.file_name().map(|name| name.to_string_lossy().starts_with('.')).unwrap_or(false);
        if !options.all_files && is_hidden {
            continue;
        }

        // Print indentation
        if !options.no_indent {
            for _ in 0..depth {
                print!("|   ");
            }
            if is_last {
                print!("    ");
            } else {
                print!("|   ");
            }
        }

        // Print file/directory name with prefix
        let prefix = if is_entry_last { "└── " } else { "├── " };
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
            if !options.dir_only {
                traverse_directory(root_path.as_ref(), &path, options, depth + 1, is_entry_last, stats)?;
            }
        } else {
            // If it's a file and the size option is set, print its size
            if !is_hidden {
                stats.1 += 1;
            }
            if options.print_size {
                let metadata = entry.metadata()?;
                let size = metadata.len();
                let size_str = format!(" ({:5}B)", size);
                print!("{}", size_str);
            }
            println!();
        }
    }

    Ok(())
}

pub fn list_directory<P: AsRef<Path>>(path: P, options: &TreeOptions) -> std::io::Result<()> {
    let current_path = path.as_ref();
    println!("{}", current_path.display());

    let mut stats = (0, 0); // (directories, files)
    // Recursively traverse the directory and print its contents
    traverse_directory(current_path, current_path, options, 1, false, &mut stats)?;

    println!("\n{} directories, {} files", stats.0, stats.1);
    Ok(())
}
