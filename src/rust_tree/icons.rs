use serde::Deserialize;
use std::{collections::HashMap, fs, path::Path};

#[derive(Deserialize)]
struct IconTheme {
    extensions: HashMap<String, String>,
    well_known: HashMap<String, String>,
    icons: HashMap<String, String>,
}

// Default icon theme embedded in the binary
const DEFAULT_ICON_THEME: &str = r#"{
  "well_known": {
    "cargo.toml": "📦",
    "package.json": "📦",
    "readme.md": "📖",
    "dockerfile": "🐳",
    "makefile": "⚙️",
    "gitignore": "🙈",
    "gitattributes": "🙈",
    "license": "📄",
    "changelog": "📝",
    "contributing": "🤝",
    "code_of_conduct": "📋"
  },
  "extensions": {
    "rs": "🦀",
    "py": "🐍",
    "js": "📜",
    "ts": "📘",
    "jsx": "⚛️",
    "tsx": "⚛️",
    "html": "🌐",
    "css": "🎨",
    "scss": "🎨",
    "sass": "🎨",
    "json": "📋",
    "xml": "📄",
    "yaml": "📄",
    "yml": "📄",
    "toml": "⚙️",
    "ini": "⚙️",
    "cfg": "⚙️",
    "conf": "⚙️",
    "md": "📖",
    "txt": "📄",
    "log": "📋",
    "sql": "🗄️",
    "sh": "🐚",
    "bash": "🐚",
    "zsh": "🐚",
    "fish": "🐟",
    "ps1": "💻",
    "bat": "💻",
    "cmd": "💻",
    "exe": "⚙️",
    "dll": "⚙️",
    "so": "⚙️",
    "dylib": "⚙️",
    "jar": "☕",
    "war": "☕",
    "ear": "☕",
    "class": "☕",
    "java": "☕",
    "kt": "🟣",
    "scala": "🔴",
    "go": "🐹",
    "php": "🐘",
    "rb": "💎",
    "pl": "🐪",
    "lua": "🌙",
    "r": "📊",
    "m": "🍎",
    "swift": "🦉",
    "dart": "🎯",
    "elm": "🌳",
    "clj": "🟢",
    "hs": "🔷",
    "ml": "🟠",
    "fs": "🔵",
    "vb": "🔵",
    "cs": "🔵",
    "cpp": "⚡",
    "c": "⚡",
    "h": "⚡",
    "hpp": "⚡",
    "cc": "⚡",
    "cxx": "⚡",
    "png": "🖼️",
    "jpg": "🖼️",
    "jpeg": "🖼️",
    "gif": "🖼️",
    "svg": "🎨",
    "ico": "🖼️",
    "bmp": "🖼️",
    "tiff": "🖼️",
    "webp": "🖼️",
    "mp4": "🎬",
    "avi": "🎬",
    "mov": "🎬",
    "wmv": "🎬",
    "flv": "🎬",
    "webm": "🎬",
    "mkv": "🎬",
    "mp3": "🎵",
    "wav": "🎵",
    "flac": "🎵",
    "aac": "🎵",
    "ogg": "🎵",
    "zip": "📦",
    "rar": "📦",
    "7z": "📦",
    "tar": "📦",
    "gz": "📦",
    "bz2": "📦",
    "xz": "📦",
    "pdf": "📕",
    "doc": "📘",
    "docx": "📘",
    "xls": "📊",
    "xlsx": "📊",
    "ppt": "📽️",
    "pptx": "📽️",
    "odt": "📄",
    "ods": "📊",
    "odp": "📽️",
    "rtf": "📄",
    "csv": "📊",
    "tsv": "📊",
    "db": "🗄️",
    "sqlite": "🗄️",
    "sqlite3": "🗄️",
    "lock": "🔒",
    "key": "🔑",
    "pem": "🔐",
    "crt": "🔐",
    "p12": "🔐",
    "pfx": "🔐"
  },
  "icons": {
    "dir": "📁",
    "file": "📄",
    "symlink": "🔗",
    "executable": "⚙️",
    "image": "🖼️",
    "video": "🎬",
    "audio": "🎵",
    "archive": "📦",
    "document": "📄",
    "spreadsheet": "📊",
    "presentation": "📽️",
    "code": "💻",
    "config": "⚙️",
    "data": "🗄️",
    "font": "🔤",
    "binary": "⚙️"
  }
}"#;

pub struct IconManager {
    theme: IconTheme,
}

impl IconManager {
    pub fn new() -> Self {
        Self {
            theme: Self::load_default_theme(),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let data = fs::read_to_string(path)?;
        let theme: IconTheme = serde_json::from_str(&data)?;
        Ok(Self { theme })
    }

    fn load_default_theme() -> IconTheme {
        serde_json::from_str(DEFAULT_ICON_THEME).unwrap()
    }

    pub fn get_icon_for_path(&self, path: &Path) -> &str {
        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
            let key = name.to_ascii_lowercase();
            if let Some(icon) = self.theme.well_known.get(&key) {
                return icon;
            }
        }

        if path.is_dir() {
            if let Some(icon) = self.theme.icons.get("dir") {
                return icon;
            }
            return "📁";
        }

        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            let key = ext.to_ascii_lowercase();
            if let Some(icon) = self.theme.extensions.get(&key) {
                return icon;
            }
        }

        if let Some(icon) = self.theme.icons.get("file") {
            return icon;
        }
        "📄"
    }
}

impl Default for IconManager {
    fn default() -> Self {
        Self::new()
    }
}
