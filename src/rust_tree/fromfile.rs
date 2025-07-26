use std::io::{self, BufRead, BufReader};
use std::fs::File;
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
        entries.insert(clean_path.to_string(), FileEntry {
            path: clean_path.to_string(),
            is_dir,
            size: None,
        });

        // Add parent directories
        let path_parts: Vec<&str> = clean_path.split('/').collect();
        for i in 1..path_parts.len() {
            let parent_path = path_parts[..i].join("/");
            if !parent_path.is_empty() {
                entries.entry(parent_path.clone()).or_insert_with(|| FileEntry {
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