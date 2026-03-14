use glob::Pattern;
use rust_tree::rust_tree::options::TreeOptions;
use rust_tree::rust_tree::traversal::list_directory_as_string;
use std::fs;
use tempfile::tempdir;

fn default_options() -> TreeOptions {
    TreeOptions {
        all_files: false,
        level: None,
        full_path: false,
        dir_only: false,
        no_indent: false,
        print_size: false,
        human_readable: false,
        pattern_glob: vec![],
        exclude_patterns: vec![],
        color: false,
        no_color: false,
        ascii: false,
        sort_by_time: false,
        reverse: false,
        print_mod_date: false,
        output_file: None,
        file_limit: None,
        dirs_first: false,
        classify: false,
        no_report: false,
        print_permissions: false,
        from_file: false,
        icons: false,
        prune: false,
        match_dirs: false,
        gitignore: false,
    }
}

fn gitignore_options() -> TreeOptions {
    let mut opts = default_options();
    opts.gitignore = true;
    opts
}

#[test]
fn root_gitignore_hides_matched_files_at_root() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join(".gitignore"), "*.log\n").unwrap();
    fs::write(p.join("app.rs"), "").unwrap();
    fs::write(p.join("debug.log"), "").unwrap();

    let mut opts = gitignore_options();
    opts.all_files = true;

    let output = list_directory_as_string(p, &opts).unwrap();
    assert!(output.contains("app.rs"), "Non-ignored file should appear");
    assert!(
        !output.contains("debug.log"),
        "Gitignored file at root should be hidden"
    );
}

#[test]
fn root_gitignore_hides_matched_files_in_subdirectories() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join(".gitignore"), "*.log\n").unwrap();
    fs::create_dir(p.join("sub")).unwrap();
    fs::write(p.join("sub").join("nested.log"), "").unwrap();
    fs::write(p.join("sub").join("keep.txt"), "").unwrap();

    let output = list_directory_as_string(p, &gitignore_options()).unwrap();
    assert!(
        !output.contains("nested.log"),
        "Gitignored file in subdirectory should be hidden"
    );
    assert!(
        output.contains("keep.txt"),
        "Non-ignored file should appear"
    );
}

#[test]
fn nested_gitignore_applies_only_to_subtree() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("sub")).unwrap();
    fs::write(p.join("sub").join(".gitignore"), "*.tmp\n").unwrap();
    fs::write(p.join("root.tmp"), "").unwrap();
    fs::write(p.join("sub").join("child.tmp"), "").unwrap();

    let mut opts = gitignore_options();
    opts.all_files = true;

    let output = list_directory_as_string(p, &opts).unwrap();
    assert!(
        output.contains("root.tmp"),
        "root.tmp should appear (nested rule does not apply at root)"
    );
    assert!(
        !output.contains("child.tmp"),
        "child.tmp should be hidden by nested .gitignore"
    );
}

#[test]
fn parent_rules_stack_with_child_rules() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join(".gitignore"), "*.log\n").unwrap();
    fs::create_dir(p.join("sub")).unwrap();
    fs::write(p.join("sub").join(".gitignore"), "*.tmp\n").unwrap();
    fs::write(p.join("sub").join("a.log"), "").unwrap();
    fs::write(p.join("sub").join("b.tmp"), "").unwrap();
    fs::write(p.join("sub").join("c.txt"), "").unwrap();

    let mut opts = gitignore_options();
    opts.all_files = true;

    let output = list_directory_as_string(p, &opts).unwrap();
    assert!(
        !output.contains("a.log"),
        "Parent rule should hide .log in child"
    );
    assert!(
        !output.contains("b.tmp"),
        "Child rule should hide .tmp in child"
    );
    assert!(output.contains("c.txt"), "Non-ignored file should appear");
}

#[test]
fn ignore_file_works_same_as_gitignore() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join(".ignore"), "secret.txt\n").unwrap();
    fs::write(p.join("secret.txt"), "").unwrap();
    fs::write(p.join("public.txt"), "").unwrap();

    let mut opts = gitignore_options();
    opts.all_files = true;

    let output = list_directory_as_string(p, &opts).unwrap();
    assert!(
        !output.contains("secret.txt"),
        ".ignore file should work like .gitignore"
    );
    assert!(output.contains("public.txt"));
}

#[test]
fn no_op_when_gitignore_flag_not_passed() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join(".gitignore"), "*.log\n").unwrap();
    fs::write(p.join("debug.log"), "").unwrap();
    fs::write(p.join("app.rs"), "").unwrap();

    let mut opts = default_options();
    opts.all_files = true;

    let output = list_directory_as_string(p, &opts).unwrap();
    assert!(
        output.contains("debug.log"),
        "Without --gitignore flag, .gitignore should have no effect"
    );
    assert!(output.contains("app.rs"));
}

#[test]
fn no_op_when_no_gitignore_file_exists() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("hello.txt"), "").unwrap();

    let output = list_directory_as_string(p, &gitignore_options()).unwrap();
    assert!(
        output.contains("hello.txt"),
        "No .gitignore file should not cause errors or hide files"
    );
}

#[test]
fn unanchored_glob_matches_at_any_depth() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join(".gitignore"), "*.log\n").unwrap();
    fs::write(p.join("root.log"), "").unwrap();
    fs::create_dir_all(p.join("a").join("b")).unwrap();
    fs::write(p.join("a").join("b").join("deep.log"), "").unwrap();
    fs::write(p.join("a").join("b").join("keep.txt"), "").unwrap();

    let mut opts = gitignore_options();
    opts.all_files = true;

    let output = list_directory_as_string(p, &opts).unwrap();
    assert!(!output.contains("root.log"));
    assert!(!output.contains("deep.log"));
    assert!(output.contains("keep.txt"));
}

#[test]
fn anchored_pattern_with_slash() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join(".gitignore"), "src/generated\n").unwrap();
    fs::create_dir_all(p.join("src").join("generated")).unwrap();
    fs::write(p.join("src").join("generated").join("root_out.rs"), "").unwrap();
    fs::create_dir_all(p.join("other").join("src").join("generated")).unwrap();
    fs::write(
        p.join("other")
            .join("src")
            .join("generated")
            .join("other_out.rs"),
        "",
    )
    .unwrap();

    let mut opts = gitignore_options();
    opts.all_files = true;

    let output = list_directory_as_string(p, &opts).unwrap();
    assert!(
        !output.contains("root_out.rs"),
        "src/generated/root_out.rs should be hidden by anchored pattern"
    );
    assert!(
        output.contains("other_out.rs"),
        "other/src/generated/other_out.rs should remain (not anchored at root)"
    );
}

#[test]
fn leading_slash_anchor_matches_only_at_root() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join(".gitignore"), "/build\n").unwrap();
    fs::create_dir(p.join("build")).unwrap();
    fs::write(p.join("build").join("out.o"), "").unwrap();
    fs::create_dir_all(p.join("src").join("build")).unwrap();
    fs::write(p.join("src").join("build").join("nested.o"), "").unwrap();

    let mut opts = gitignore_options();
    opts.all_files = true;

    let output = list_directory_as_string(p, &opts).unwrap();
    assert!(
        output.contains("nested.o"),
        "src/build/nested.o should remain (not anchored at root)"
    );
    let lines: Vec<&str> = output.lines().collect();
    let has_root_build_content = lines.iter().any(|l| l.contains("out.o"));
    assert!(!has_root_build_content, "Root build/out.o should be hidden");
}

#[test]
fn dir_only_pattern_matches_directories_not_files() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join(".gitignore"), "build/\n").unwrap();
    fs::create_dir(p.join("build")).unwrap();
    fs::write(p.join("build").join("out.o"), "").unwrap();
    fs::write(p.join("sub_build"), "").unwrap();

    let mut opts = gitignore_options();
    opts.all_files = true;

    let output = list_directory_as_string(p, &opts).unwrap();
    assert!(
        !output.contains("out.o"),
        "Contents of build/ dir should be hidden"
    );
    assert!(
        output.contains("sub_build"),
        "File named sub_build should not be hidden by dir-only pattern"
    );
}

#[test]
fn negation_overrides_previous_rule() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join(".gitignore"), "*.log\n!important.log\n").unwrap();
    fs::write(p.join("debug.log"), "").unwrap();
    fs::write(p.join("important.log"), "").unwrap();

    let mut opts = gitignore_options();
    opts.all_files = true;

    let output = list_directory_as_string(p, &opts).unwrap();
    assert!(!output.contains("debug.log"), "debug.log should be hidden");
    assert!(
        output.contains("important.log"),
        "important.log should be un-ignored by negation"
    );
}

#[test]
fn negation_order_last_match_wins() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(
        p.join(".gitignore"),
        "*.log\n!important.log\nimportant.log\n",
    )
    .unwrap();
    fs::write(p.join("important.log"), "").unwrap();

    let mut opts = gitignore_options();
    opts.all_files = true;

    let output = list_directory_as_string(p, &opts).unwrap();
    assert!(
        !output.contains("important.log"),
        "Last match wins: re-ignoring important.log should hide it"
    );
}

#[test]
fn parent_dir_exclusion_blocks_child_negation() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join(".gitignore"), "build/\n!build/output.txt\n").unwrap();
    fs::create_dir(p.join("build")).unwrap();
    fs::write(p.join("build").join("output.txt"), "").unwrap();
    fs::write(p.join("build").join("other.o"), "").unwrap();

    let mut opts = gitignore_options();
    opts.all_files = true;

    let output = list_directory_as_string(p, &opts).unwrap();
    assert!(
        !output.contains("output.txt"),
        "Cannot un-ignore file inside excluded parent directory"
    );
    assert!(!output.contains("other.o"));
}

#[test]
fn prune_plus_gitignore() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join(".gitignore"), "*.log\n").unwrap();
    fs::create_dir(p.join("logs_only")).unwrap();
    fs::write(p.join("logs_only").join("app.log"), "").unwrap();
    fs::create_dir(p.join("mixed")).unwrap();
    fs::write(p.join("mixed").join("code.rs"), "").unwrap();
    fs::write(p.join("mixed").join("debug.log"), "").unwrap();

    let mut opts = gitignore_options();
    opts.all_files = true;
    opts.pattern_glob = vec![Pattern::new("*.rs").unwrap()];
    opts.prune = true;

    let output = list_directory_as_string(p, &opts).unwrap();
    assert!(
        output.contains("mixed"),
        "mixed dir has .rs files, should survive prune"
    );
    assert!(output.contains("code.rs"));
    assert!(
        !output.contains("logs_only"),
        "logs_only has only .log files (gitignored), should be pruned"
    );
}

#[test]
fn pattern_plus_gitignore() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join(".gitignore"), "*.log\n").unwrap();
    fs::write(p.join("app.log"), "").unwrap();
    fs::write(p.join("code.rs"), "").unwrap();
    fs::write(p.join("data.txt"), "").unwrap();

    let mut opts = gitignore_options();
    opts.all_files = true;
    opts.pattern_glob = vec![Pattern::new("*.log").unwrap()];

    let output = list_directory_as_string(p, &opts).unwrap();
    assert!(
        !output.contains("app.log"),
        "Gitignored files excluded despite matching -P pattern"
    );
}

#[test]
fn matchdirs_plus_gitignore() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join(".gitignore"), "*.tmp\n").unwrap();
    fs::create_dir_all(p.join("mymod")).unwrap();
    fs::write(p.join("mymod").join("code.rs"), "").unwrap();
    fs::write(p.join("mymod").join("cache.tmp"), "").unwrap();

    let mut opts = gitignore_options();
    opts.all_files = true;
    opts.pattern_glob = vec![Pattern::new("mymod").unwrap()];
    opts.match_dirs = true;

    let output = list_directory_as_string(p, &opts).unwrap();
    assert!(output.contains("mymod"));
    assert!(output.contains("code.rs"), "Non-ignored child should show");
    assert!(
        !output.contains("cache.tmp"),
        "Gitignored files hidden inside matched dirs"
    );
}

#[test]
fn filelimit_plus_gitignore_uses_raw_count() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join(".gitignore"), "*.log\n").unwrap();
    fs::create_dir(p.join("big")).unwrap();
    fs::write(p.join("big").join("a.log"), "").unwrap();
    fs::write(p.join("big").join("b.log"), "").unwrap();
    fs::write(p.join("big").join("c.rs"), "").unwrap();

    fs::create_dir(p.join("small")).unwrap();
    fs::write(p.join("small").join("d.rs"), "").unwrap();
    fs::write(p.join("small").join("e.log"), "").unwrap();

    let mut opts = gitignore_options();
    opts.all_files = true;
    opts.file_limit = Some(2);

    let output = list_directory_as_string(p, &opts).unwrap();
    assert!(
        !output.contains("c.rs"),
        "big/ should be excluded by filelimit (raw count 3 > 2)"
    );
    assert!(output.contains("d.rs"), "small/ should be within filelimit");
}

#[test]
fn hidden_plus_gitignore() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join(".gitignore"), ".secret\n").unwrap();
    fs::write(p.join(".hidden"), "").unwrap();
    fs::write(p.join(".secret"), "").unwrap();
    fs::write(p.join("visible.txt"), "").unwrap();

    let mut opts = gitignore_options();
    opts.all_files = true;

    let output = list_directory_as_string(p, &opts).unwrap();
    assert!(
        output.contains(".hidden"),
        "Hidden files shown with -a unless gitignored"
    );
    assert!(
        !output.contains(".secret"),
        ".secret should be hidden by gitignore"
    );
    assert!(output.contains("visible.txt"));
}

#[test]
fn dir_only_plus_gitignore() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join(".gitignore"), "build/\n").unwrap();
    fs::create_dir(p.join("src")).unwrap();
    fs::create_dir(p.join("build")).unwrap();
    fs::write(p.join("src").join("main.rs"), "").unwrap();
    fs::write(p.join("build").join("out.o"), "").unwrap();

    let mut opts = gitignore_options();
    opts.all_files = true;
    opts.dir_only = true;

    let output = list_directory_as_string(p, &opts).unwrap();
    assert!(output.contains("src"), "Non-ignored dir should appear");
    assert!(
        !output.contains("build"),
        "Dir-only gitignore pattern should hide build/ in -d mode"
    );
}

#[test]
fn exclude_plus_gitignore_compose() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join(".gitignore"), "*.log\n").unwrap();
    fs::write(p.join("debug.log"), "").unwrap();
    fs::write(p.join("readme.txt"), "").unwrap();
    fs::write(p.join("notes.txt"), "").unwrap();
    fs::write(p.join("code.rs"), "").unwrap();

    let mut opts = gitignore_options();
    opts.all_files = true;
    opts.exclude_patterns = vec![Pattern::new("*.txt").unwrap()];

    let output = list_directory_as_string(p, &opts).unwrap();
    assert!(
        !output.contains("debug.log"),
        "Gitignore should hide .log files"
    );
    assert!(
        !output.contains("readme.txt"),
        "Exclude should hide .txt files"
    );
    assert!(
        !output.contains("notes.txt"),
        "Exclude should hide .txt files"
    );
    assert!(
        output.contains("code.rs"),
        "Non-excluded file should appear"
    );
}

#[test]
fn nested_gitignore_anchored_pattern_matches_in_subtree() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    // Root has no .gitignore
    fs::create_dir(p.join("sub")).unwrap();
    // Nested .gitignore with anchored pattern "/config"
    fs::write(p.join("sub").join(".gitignore"), "/config\n").unwrap();

    // sub/config should be ignored (anchored to sub/)
    fs::create_dir(p.join("sub").join("config")).unwrap();
    fs::write(p.join("sub").join("config").join("settings.json"), "").unwrap();
    fs::write(p.join("sub").join("code.rs"), "").unwrap();

    // root-level config should NOT be ignored (rule is in sub/.gitignore)
    fs::create_dir(p.join("config")).unwrap();
    fs::write(p.join("config").join("global.json"), "").unwrap();

    let mut opts = gitignore_options();
    opts.all_files = true;

    let output = list_directory_as_string(p, &opts).unwrap();
    assert!(
        output.contains("code.rs"),
        "Non-ignored file in sub/ should appear"
    );
    assert!(
        output.contains("global.json"),
        "Root-level config/global.json should appear (rule is scoped to sub/)"
    );
    assert!(
        !output.contains("settings.json"),
        "sub/config/settings.json should be hidden by nested anchored /config pattern"
    );
}

#[test]
fn fromfile_plus_gitignore_silently_ignored() {
    let dir = tempdir().unwrap();
    let listing_file = dir.path().join("listing.txt");

    let content = "src/\nsrc/main.rs\nbuild/\nbuild/out.o\n";
    fs::write(&listing_file, content).unwrap();

    let mut opts = default_options();
    opts.from_file = true;
    opts.gitignore = true;

    let result = list_directory_as_string(&listing_file, &opts);
    assert!(result.is_ok(), "fromfile + gitignore should not crash");
    let output = result.unwrap();
    assert!(output.contains("main.rs"));
    assert!(output.contains("out.o"));
}
