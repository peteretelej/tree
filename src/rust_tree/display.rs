use ansi_term::Colour::{Blue, Cyan, Green, Red, Yellow};
use is_executable::IsExecutable;
use std::fs;

pub fn colorize(entry: &fs::DirEntry, text: String) -> String {
    let file_type = entry.file_type().unwrap();
    let is_exec = entry.path().is_executable();

    if file_type.is_dir() {
        Blue.bold().paint(text).to_string()
    } else if file_type.is_symlink() {
        Cyan.paint(text).to_string()
    } else if is_exec {
        Green.paint(text).to_string()
    } else if let Some(extension) = entry.path().extension() {
        match extension.to_string_lossy().to_lowercase().as_str() {
            "tar" | "gz" | "xz" | "bz2" | "zip" | "7z" => Red.paint(text).to_string(),
            "jpg" | "jpeg" | "bmp" | "gif" | "png" => Yellow.paint(text).to_string(),
            _ => text,
        }
    } else {
        text
    }
}
