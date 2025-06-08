use ansi_term::Colour::{Blue, Cyan, Green, Red, Yellow};
use is_executable::IsExecutable;
use std::fs;

pub fn colorize(entry: &fs::DirEntry, text: &str) -> String {
    let file_type = match entry.file_type() {
        Ok(ft) => ft,
        Err(_) => return text.to_string(), // Return original text if file type fails
    };

    // Use is_executable crate for cross-platform check
    let is_exec = entry.path().is_executable();

    if file_type.is_dir() {
        Blue.bold().paint(text).to_string()
    } else if file_type.is_symlink() {
        Cyan.paint(text).to_string()
    } else if is_exec {
        Green.paint(text).to_string()
    } else if let Some(extension) = entry.path().extension().and_then(|ext| ext.to_str()) {
        // Convert extension to lowercase once
        match extension.to_lowercase().as_str() {
            // Archives
            "tar" | "gz" | "xz" | "bz2" | "zip" | "7z" => Red.paint(text).to_string(),
            // Images
            "jpg" | "jpeg" | "bmp" | "gif" | "png" => Yellow.paint(text).to_string(),
            // Default: no color for other extensions
            _ => text.to_string(),
        }
    } else {
        // Default: no color if no extension or invalid UTF-8
        text.to_string()
    }
}

// --- Permissions Formatting (Unix only) ---
#[cfg(unix)]
pub fn format_permissions_unix(mode: u32, is_dir: bool) -> String {
    let mut perms = String::with_capacity(10); // [drwxrwxrwx]

    perms.push(if is_dir { 'd' } else { '-' });

    // Owner permissions
    perms.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o100 != 0 { 'x' } else { '-' }); // Consider setuid/setgid later?

    // Group permissions
    perms.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o010 != 0 { 'x' } else { '-' }); // Consider setgid later?

    // Other permissions
    perms.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o001 != 0 { 'x' } else { '-' }); // Consider sticky bit later?

    format!("[{}]", perms)
}

// Stub for non-Unix platforms
#[cfg(not(unix))]
pub fn format_permissions_unix(_mode: u32, _is_dir: bool) -> String {
    String::new() // No permissions string on non-Unix
}
