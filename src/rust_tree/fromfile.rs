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
            FileListFormat::Zip => parse_zip_listing(lines),
            FileListFormat::SevenZip => parse_7z_listing(lines),
            FileListFormat::Rar => parse_rar_listing(lines),
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
    Zip,
    SevenZip,
    Rar,
}

fn detect_format(lines: &[String]) -> Option<FileListFormat> {
    if lines.is_empty() {
        return None;
    }

    // Check for RAR format patterns first
    let rar_patterns = lines
        .iter()
        .take(10)
        .filter(|line| {
            // rar l/v format: has "RAR" header, "Archive:", "Details: RAR", or attribute patterns
            line.starts_with("RAR ")
                || line.starts_with("Archive:")
                || line.starts_with("Details: RAR")
                || line.contains("Attributes")
                || (line.starts_with(" -") && line.contains("--") && line.len() > 40)
                || (line.starts_with(" d") && line.contains("--") && line.len() > 40)
        })
        .count();

    if rar_patterns > 0 {
        return Some(FileListFormat::Rar);
    }

    // Check for 7-Zip format patterns
    let sevenz_patterns = lines
        .iter()
        .take(10)
        .filter(|line| {
            // 7z l format: has "7-Zip" header, "Listing archive:", or specific patterns
            line.starts_with("7-Zip")
                || line.starts_with("Listing archive:")
                || line.starts_with("Path =")
                || line.starts_with("Type =")
                || (line.contains("D....") || line.contains("....A"))
        })
        .count();

    if sevenz_patterns > 0 {
        return Some(FileListFormat::SevenZip);
    }

    // Check for ZIP format patterns
    let zip_patterns = lines
        .iter()
        .take(10)
        .filter(|line| {
            // unzip -l format: has "Archive:" header or dashed separators
            line.starts_with("Archive:")
                || line.starts_with("---------")
                || (line.contains(" ")
                    && line.contains(":")
                    && line.len() > 40
                    && line.split_whitespace().count() >= 4)
        })
        .count();

    if zip_patterns > 0 {
        return Some(FileListFormat::Zip);
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

fn parse_zip_listing(lines: Vec<String>) -> Vec<FileEntry> {
    let mut entries = std::collections::HashMap::new();

    for line in lines {
        if let Some(entry) = parse_zip_line(&line) {
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

fn parse_zip_line(line: &str) -> Option<FileEntry> {
    let line = line.trim();
    if line.is_empty()
        || line.starts_with("Archive:")
        || line.starts_with("Length")
        || line.starts_with("---------")
        || line.ends_with("files")
        || line.starts_with("Method")
        || line.starts_with("------")
    {
        return None;
    }

    // Check if this is a verbose format line by looking for specific patterns
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 8 && parts[1] == "Stored" || parts.get(1) == Some(&"Deflated") {
        // unzip -v format: "       9  Stored        9   0% 2025-07-26 19:41 433ac1df  test_zip/file1.txt"
        parse_zip_verbose_line(line)
    } else if parts.len() >= 4 && parts[0].parse::<u64>().is_ok() {
        // unzip -l format: "        9  2025-07-26 19:41   test_zip/file1.txt"
        parse_zip_simple_line(line)
    } else {
        None
    }
}

fn parse_zip_simple_line(line: &str) -> Option<FileEntry> {
    // unzip -l format: "        9  2025-07-26 19:41   test_zip/file1.txt"
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 4 {
        return None;
    }

    let size = parts[0].parse::<u64>().ok();
    // Date and time are at parts[1] and parts[2]
    // Path starts at parts[3] and may continue if there are spaces
    let path = parts[3..].join(" ");

    let is_dir = path.ends_with('/');
    let clean_path = if is_dir {
        path.trim_end_matches('/')
    } else {
        &path
    };

    Some(FileEntry {
        path: clean_path.to_string(),
        is_dir,
        size,
    })
}

fn parse_zip_verbose_line(line: &str) -> Option<FileEntry> {
    // unzip -v format: "       9  Stored        9   0% 2025-07-26 19:41 433ac1df  test_zip/file1.txt"
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 7 {
        return None;
    }

    let size = parts[0].parse::<u64>().ok();
    // Path is the last part
    let path = parts[parts.len() - 1];

    let is_dir = path.ends_with('/');
    let clean_path = if is_dir {
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

fn parse_7z_listing(lines: Vec<String>) -> Vec<FileEntry> {
    let mut entries = std::collections::HashMap::new();

    for line in lines {
        if let Some(entry) = parse_7z_line(&line) {
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

fn parse_7z_line(line: &str) -> Option<FileEntry> {
    let line = line.trim();
    if line.is_empty()
        || line.starts_with("7-Zip")
        || line.starts_with("p7zip")
        || line.starts_with("Scanning")
        || line.starts_with("Creating")
        || line.starts_with("Items to")
        || line.starts_with("Files read")
        || line.starts_with("Archive size")
        || line.starts_with("Everything")
        || line.starts_with("Listing archive:")
        || line.starts_with("--")
        || line.starts_with("Path =")
        || line.starts_with("Type =")
        || line.starts_with("Physical Size")
        || line.starts_with("Headers Size")
        || line.starts_with("Method =")
        || line.starts_with("Solid =")
        || line.starts_with("Blocks =")
        || line.starts_with("Date")
        || line.starts_with("---")
        || line.contains("files,")
        || line.contains("bytes (")
        || line.contains("file,")
    {
        return None;
    }

    // 7z format: "2025-07-26 19:58:52 D....            0            0  test_7z"
    // or:        "2025-07-26 19:58:52 ....A            9               test_7z/file1.txt"
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 5 {
        return None;
    }

    // Check if this looks like a valid 7z entry (starts with date)
    if !parts[0].contains('-') || !parts[1].contains(':') {
        return None;
    }

    // Skip summary lines (no attr field)
    if parts.len() >= 3 && !parts[2].contains('.') {
        return None;
    }

    // Date: parts[0], Time: parts[1], Attr: parts[2], Size: parts[3], Compressed: parts[4] (optional), Name: parts[5..] or parts[4..]
    let attr = parts[2];
    let is_dir = attr.starts_with('D');

    let size = if !is_dir && parts.len() > 3 {
        parts[3].parse::<u64>().ok()
    } else {
        None
    };

    // Path is from the last field(s) - handle both formats
    let path = if parts.len() >= 6 {
        // Format: date time attr size compressed name
        parts[5..].join(" ")
    } else if parts.len() >= 5 {
        // Format: date time attr size name (no compressed size)
        parts[4..].join(" ")
    } else {
        return None;
    };

    Some(FileEntry {
        path: path.to_string(),
        is_dir,
        size,
    })
}

fn parse_rar_listing(lines: Vec<String>) -> Vec<FileEntry> {
    let mut entries = std::collections::HashMap::new();

    for line in lines {
        if let Some(entry) = parse_rar_line(&line) {
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

fn parse_rar_line(line: &str) -> Option<FileEntry> {
    let line = line.trim();
    if line.is_empty()
        || line.starts_with("RAR ")
        || line.starts_with("Trial version")
        || line.starts_with("Evaluation copy")
        || line.starts_with("Archive:")
        || line.starts_with("Details:")
        || line.starts_with("Attributes")
        || line.starts_with("-----------")
        || line.starts_with("Creating")
        || line.starts_with("Adding")
        || line.starts_with("Done")
        || line
            .chars()
            .all(|c| c.is_whitespace() || c.is_ascii_digit())
        || (line.len() < 30
            && line
                .chars()
                .all(|c| c.is_whitespace() || c.is_ascii_digit() || c == '%'))
    {
        return None;
    }

    // RAR format: " -rw-rw-r--         9  2025-07-26 21:18  test_rar/file1.txt"
    // RAR verbose: " -rw-rw-r--         9         9 100%  2025-07-26 21:18  3E4D359A  test_rar/file1.txt"
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 5 {
        return None;
    }

    // First field should be attributes/permissions
    let attr = parts[0];
    if !attr.starts_with('-') && !attr.starts_with('d') && !attr.starts_with('l') {
        return None;
    }

    let is_dir = attr.starts_with('d');

    // Size is the second field
    let size = if !is_dir && parts.len() > 1 {
        parts[1].parse::<u64>().ok()
    } else {
        None
    };

    // Determine if this is verbose format (has packed size and ratio)
    let path = if parts.len() >= 8 && parts[3].ends_with('%') {
        // Verbose format: attr size packed ratio date time checksum name
        parts[7..].join(" ")
    } else if parts.len() >= 5 {
        // Simple format: attr size date time name
        parts[4..].join(" ")
    } else {
        return None;
    };

    Some(FileEntry {
        path: path.to_string(),
        is_dir,
        size,
    })
}

pub fn build_virtual_tree(entries: Vec<FileEntry>) -> VirtualTree {
    VirtualTree {
        entries,
        root_name: ".".to_string(),
    }
}
