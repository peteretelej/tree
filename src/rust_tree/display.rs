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
