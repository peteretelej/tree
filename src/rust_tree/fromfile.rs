use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
}

pub struct VirtualTree {
    pub entries: Vec<FileEntry>,
    pub root_name: String,
}

pub fn read_file_listing<P: AsRef<Path>>(input_path: P) -> io::Result<Vec<String>> {
    let path = input_path.as_ref();
    let reader: Box<dyn BufRead> = if path.to_str() == Some(".") {
        Box::new(BufReader::new(io::stdin()))
    } else {
        Box::new(BufReader::new(File::open(path)?))
    };

    let mut lines = Vec::new();
    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            lines.push(trimmed.to_string());
        }
    }
    Ok(lines)
}

pub fn parse_file_listing(lines: Vec<String>) -> Vec<FileEntry> {
    if let Some(format) = detect_format(&lines) {
        match format {
            FileListFormat::Tar => parse_tar_listing(lines),
            FileListFormat::Simple => parse_simple_paths(lines),
        }
    } else {
        parse_simple_paths(lines)
    }
}

#[derive(Debug)]
enum FileListFormat {
    Simple,
    Tar,
}

fn detect_format(lines: &[String]) -> Option<FileListFormat> {
    if lines.is_empty() {
        return None;
    }

    // Check for tar -tvf format patterns (more specific)
    let tar_patterns = lines
        .iter()
        .take(5)
        .filter(|line| {
            // tar -tvf format: "drwxr-xr-x user/group size date path"
            // Must have at least 6 space-separated parts and start with permission pattern
            let parts: Vec<&str> = line.split_whitespace().collect();
            parts.len() >= 6
                && (line.starts_with("drwx")
                    || line.starts_with("-rw")
                    || line.starts_with("lrwx")
                    || line.starts_with("-rwx"))
        })
        .count();

    if tar_patterns > 0 {
        Some(FileListFormat::Tar)
    } else {
        Some(FileListFormat::Simple)
    }
}

fn parse_tar_listing(lines: Vec<String>) -> Vec<FileEntry> {
    let mut entries = std::collections::HashMap::new();

    for line in lines {
        if let Some(entry) = parse_tar_line(&line) {
            if entry.path.is_empty() {
                continue;
            }

            // Add the entry itself
            entries.insert(entry.path.clone(), entry.clone());

            // Add parent directories
            let path_parts: Vec<&str> = entry.path.split('/').collect();
            for i in 1..path_parts.len() {
                let parent_path = path_parts[..i].join("/");
                if !parent_path.is_empty() {
                    entries
                        .entry(parent_path.clone())
                        .or_insert_with(|| FileEntry {
                            path: parent_path,
                            is_dir: true,
                            size: None,
                        });
                }
            }
        }
    }

    entries.into_values().collect()
}

fn parse_tar_line(line: &str) -> Option<FileEntry> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    // Handle both tar -tf and tar -tvf formats
    if line.starts_with('d') || line.starts_with('-') || line.starts_with('l') {
        // tar -tvf format: "drwxr-xr-x user/group 0 date path"
        parse_tar_verbose_line(line)
    } else {
        // tar -tf format: just the path
        parse_tar_simple_line(line)
    }
}

fn parse_tar_verbose_line(line: &str) -> Option<FileEntry> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 6 {
        return None;
    }

    let permissions = parts[0];
    let is_dir = permissions.starts_with('d');

    // Size is typically at index 2
    let size = if parts.len() > 2 {
        parts[2].parse::<u64>().ok()
    } else {
        None
    };

    // Path is the last part (may contain spaces, so rejoin)
    let path_start = line.rfind(' ')?;
    let path = &line[path_start + 1..];

    let clean_path = if is_dir && path.ends_with('/') {
        path.trim_end_matches('/')
    } else {
        path
    };

    Some(FileEntry {
        path: clean_path.to_string(),
        is_dir,
        size,
    })
}

fn parse_tar_simple_line(line: &str) -> Option<FileEntry> {
    let is_dir = line.ends_with('/');
    let clean_path = if is_dir {
        line.trim_end_matches('/')
    } else {
        line
    };

    if clean_path.is_empty() {
        return None;
    }

    Some(FileEntry {
        path: clean_path.to_string(),
        is_dir,
        size: None,
    })
}

pub fn parse_simple_paths(lines: Vec<String>) -> Vec<FileEntry> {
    let mut entries = std::collections::HashMap::new();

    for line in lines {
        let is_dir = line.ends_with('/');
        let clean_path = if is_dir {
            line.trim_end_matches('/')
        } else {
            &line
        };

        if clean_path.is_empty() {
            continue;
        }

        // Add the entry itself
        entries.insert(
            clean_path.to_string(),
            FileEntry {
                path: clean_path.to_string(),
                is_dir,
                size: None,
            },
        );

        // Add parent directories
        let path_parts: Vec<&str> = clean_path.split('/').collect();
        for i in 1..path_parts.len() {
            let parent_path = path_parts[..i].join("/");
            if !parent_path.is_empty() {
                entries
                    .entry(parent_path.clone())
                    .or_insert_with(|| FileEntry {
                        path: parent_path,
                        is_dir: true,
                        size: None,
                    });
            }
        }
    }

    entries.into_values().collect()
}

pub fn build_virtual_tree(entries: Vec<FileEntry>) -> VirtualTree {
    VirtualTree {
        entries,
        root_name: ".".to_string(),
    }
}
