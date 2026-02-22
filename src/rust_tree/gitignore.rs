use glob::{MatchOptions, Pattern};
use std::fs;
use std::path::{Path, PathBuf};

const MATCH_OPTIONS: MatchOptions = MatchOptions {
    case_sensitive: true,
    require_literal_separator: true,
    require_literal_leading_dot: false,
};

struct GitignoreRule {
    pattern: Pattern,
    negated: bool,
    dir_only: bool,
    anchored: bool,
    source_dir: PathBuf,
}

pub struct GitignoreRules {
    rules: Vec<GitignoreRule>,
}

fn parse_line(line: &str, source_dir: &Path) -> Option<GitignoreRule> {
    let trimmed = line.trim_end();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }

    let mut s = trimmed.to_string();

    let negated = s.starts_with('!');
    if negated {
        s = s[1..].to_string();
    }

    let dir_only = s.ends_with('/');
    if dir_only {
        s = s[..s.len() - 1].to_string();
    }

    let has_leading_slash = s.starts_with('/');
    if has_leading_slash {
        s = s[1..].to_string();
    }

    let anchored = has_leading_slash || s.contains('/');

    if !anchored {
        s = format!("**/{s}");
    }

    let pattern = Pattern::new(&s).ok()?;

    Some(GitignoreRule {
        pattern,
        negated,
        dir_only,
        anchored,
        source_dir: source_dir.to_path_buf(),
    })
}

fn load_ignore_file(path: &Path, source_dir: &Path) -> Vec<GitignoreRule> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    content
        .lines()
        .filter_map(|line| parse_line(line, source_dir))
        .collect()
}

impl Default for GitignoreRules {
    fn default() -> Self {
        Self::new()
    }
}

impl GitignoreRules {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn load_for_root(root_path: &Path) -> Self {
        let mut rules = Vec::new();
        for name in &[".gitignore", ".ignore"] {
            let file_path = root_path.join(name);
            rules.extend(load_ignore_file(&file_path, root_path));
        }
        Self { rules }
    }

    pub fn extend_with_dir(&self, dir_path: &Path) -> Self {
        let mut rules = self.rules.iter().map(|r| GitignoreRule {
            pattern: r.pattern.clone(),
            negated: r.negated,
            dir_only: r.dir_only,
            anchored: r.anchored,
            source_dir: r.source_dir.clone(),
        }).collect::<Vec<_>>();
        for name in &[".gitignore", ".ignore"] {
            let file_path = dir_path.join(name);
            rules.extend(load_ignore_file(&file_path, dir_path));
        }
        Self { rules }
    }

    pub fn is_ignored(&self, rel_path: &Path, is_dir: bool) -> bool {
        let mut ignored = false;
        for rule in &self.rules {
            if rule.dir_only && !is_dir {
                continue;
            }
            let match_path = if rule.anchored {
                match rel_path.strip_prefix(&rule.source_dir) {
                    Ok(p) => p,
                    Err(_) => rel_path,
                }
            } else {
                rel_path
            };
            if rule.pattern.matches_path_with(match_path, MATCH_OPTIONS) {
                ignored = !rule.negated;
            }
        }
        ignored
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_star_star_matches_zero_segments() {
        assert!(Pattern::new("**/*.log")
            .unwrap()
            .matches_path_with(Path::new("foo.log"), MATCH_OPTIONS));
        assert!(Pattern::new("**/build")
            .unwrap()
            .matches_path_with(Path::new("build"), MATCH_OPTIONS));
    }

    #[test]
    fn glob_star_star_matches_multiple_segments() {
        assert!(Pattern::new("**/*.log")
            .unwrap()
            .matches_path_with(Path::new("a/b/foo.log"), MATCH_OPTIONS));
        assert!(Pattern::new("**/build")
            .unwrap()
            .matches_path_with(Path::new("src/build"), MATCH_OPTIONS));
    }

    #[test]
    fn parse_simple_pattern() {
        let rule = parse_line("*.log", Path::new("")).unwrap();
        assert!(!rule.negated);
        assert!(!rule.dir_only);
        assert!(!rule.anchored);
        assert!(rule.pattern.matches_path_with(Path::new("foo.log"), MATCH_OPTIONS));
        assert!(rule.pattern.matches_path_with(Path::new("a/b/foo.log"), MATCH_OPTIONS));
    }

    #[test]
    fn parse_dir_only_pattern() {
        let rule = parse_line("build/", Path::new("")).unwrap();
        assert!(!rule.negated);
        assert!(rule.dir_only);
        assert!(!rule.anchored);
        assert!(rule.pattern.matches_path_with(Path::new("build"), MATCH_OPTIONS));
    }

    #[test]
    fn parse_anchored_leading_slash() {
        let rule = parse_line("/build", Path::new("")).unwrap();
        assert!(!rule.negated);
        assert!(!rule.dir_only);
        assert!(rule.anchored);
        assert!(rule.pattern.matches_path_with(Path::new("build"), MATCH_OPTIONS));
        assert!(!rule.pattern.matches_path_with(Path::new("src/build"), MATCH_OPTIONS));
    }

    #[test]
    fn parse_anchored_contains_slash() {
        let rule = parse_line("src/generated", Path::new("")).unwrap();
        assert!(rule.anchored);
        assert!(rule.pattern.matches_path_with(Path::new("src/generated"), MATCH_OPTIONS));
        assert!(!rule.pattern.matches_path_with(Path::new("other/src/generated"), MATCH_OPTIONS));
    }

    #[test]
    fn parse_negated_pattern() {
        let rule = parse_line("!*.log", Path::new("")).unwrap();
        assert!(rule.negated);
        assert!(!rule.dir_only);
        assert!(!rule.anchored);
    }

    #[test]
    fn parse_negated_dir_only() {
        let rule = parse_line("!build/", Path::new("")).unwrap();
        assert!(rule.negated);
        assert!(rule.dir_only);
    }

    #[test]
    fn parse_comment_and_blank_lines() {
        assert!(parse_line("# comment", Path::new("")).is_none());
        assert!(parse_line("", Path::new("")).is_none());
        assert!(parse_line("   ", Path::new("")).is_none());
    }

    #[test]
    fn parse_trailing_whitespace_stripped() {
        let rule = parse_line("*.log   ", Path::new("")).unwrap();
        assert!(rule.pattern.matches_path_with(Path::new("foo.log"), MATCH_OPTIONS));
        assert!(!rule.pattern.matches_path_with(Path::new("foo.log   "), MATCH_OPTIONS));
    }

    #[test]
    fn is_ignored_last_match_wins() {
        let rules = GitignoreRules {
            rules: vec![
                parse_line("*.log", Path::new("")).unwrap(),
                parse_line("!important.log", Path::new("")).unwrap(),
            ],
        };
        assert!(rules.is_ignored(Path::new("debug.log"), false));
        assert!(!rules.is_ignored(Path::new("important.log"), false));
    }

    #[test]
    fn is_ignored_dir_only_skips_files() {
        let rules = GitignoreRules {
            rules: vec![parse_line("build/", Path::new("")).unwrap()],
        };
        assert!(rules.is_ignored(Path::new("build"), true));
        assert!(!rules.is_ignored(Path::new("build"), false));
    }

    #[test]
    fn is_ignored_anchored_relative_to_source_dir() {
        let rules = GitignoreRules {
            rules: vec![parse_line("src/generated", Path::new("project")).unwrap()],
        };
        assert!(rules.is_ignored(Path::new("project/src/generated"), false));
        assert!(!rules.is_ignored(Path::new("other/src/generated"), false));
    }

    #[test]
    fn is_empty_with_no_rules() {
        let rules = GitignoreRules::new();
        assert!(rules.is_empty());
    }

    #[test]
    fn is_empty_with_rules() {
        let rules = GitignoreRules {
            rules: vec![parse_line("*.log", Path::new("")).unwrap()],
        };
        assert!(!rules.is_empty());
    }

    #[test]
    fn load_for_root_with_temp_dir() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".gitignore"), "*.log\nbuild/\n").unwrap();
        let rules = GitignoreRules::load_for_root(dir.path());
        assert_eq!(rules.rules.len(), 2);
        assert!(rules.is_ignored(Path::new("foo.log"), false));
        assert!(rules.is_ignored(Path::new("build"), true));
        assert!(!rules.is_ignored(Path::new("build"), false));
    }

    #[test]
    fn load_for_root_reads_ignore_file() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".ignore"), "secret/\n").unwrap();
        let rules = GitignoreRules::load_for_root(dir.path());
        assert_eq!(rules.rules.len(), 1);
        assert!(rules.is_ignored(Path::new("secret"), true));
    }

    #[test]
    fn extend_with_dir_adds_child_rules() {
        let dir = tempfile::tempdir().unwrap();
        let child = dir.path().join("sub");
        fs::create_dir(&child).unwrap();
        fs::write(dir.path().join(".gitignore"), "*.log\n").unwrap();
        fs::write(child.join(".gitignore"), "*.tmp\n").unwrap();

        let root_rules = GitignoreRules::load_for_root(dir.path());
        assert_eq!(root_rules.rules.len(), 1);

        let extended = root_rules.extend_with_dir(&child);
        assert_eq!(extended.rules.len(), 2);
        assert!(extended.is_ignored(Path::new("foo.log"), false));
        assert!(extended.is_ignored(Path::new("bar.tmp"), false));
    }

    #[test]
    fn load_for_root_no_files_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let rules = GitignoreRules::load_for_root(dir.path());
        assert!(rules.is_empty());
    }

    #[test]
    fn complex_negation_scenario() {
        let rules = GitignoreRules {
            rules: vec![
                parse_line("*.log", Path::new("")).unwrap(),
                parse_line("!important.log", Path::new("")).unwrap(),
                parse_line("important.log", Path::new("")).unwrap(),
            ],
        };
        assert!(rules.is_ignored(Path::new("important.log"), false));
        assert!(rules.is_ignored(Path::new("debug.log"), false));
    }

    #[test]
    fn anchored_dir_only_combined() {
        let rule = parse_line("src/generated/", Path::new("")).unwrap();
        assert!(rule.anchored);
        assert!(rule.dir_only);
        assert!(rule.pattern.matches_path_with(Path::new("src/generated"), MATCH_OPTIONS));
    }
}
