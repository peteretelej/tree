//! Table-driven harnesses covering both renderers at once.
//!
//! `tree` has two independent renderers: one walks a real directory, the other
//! renders a listing supplied via `--fromfile`. They share only `TreeOptions`
//! and no formatting code, so every flag must be wired twice and nothing fails
//! when the second wiring is forgotten. Issue #68 (`--color`) was one symptom;
//! `-r`, `--filelimit` and `--icons` were silently ignored the same way.
//!
//! Rather than hand-writing a `--fromfile` test per flag, these two harnesses
//! assert the two properties that actually matter:
//!
//! * [`flag_changes_output`] - the flag does *something* in each renderer.
//!   Catches silent no-ops.
//! * [`renderers_agree`] - the flag does the *same* thing in both renderers.
//!   Catches divergence.
//!
//! Neither is sufficient alone. Parity passes when both renderers are wrong in
//! the same way; "changes output" passes when the change itself is wrong. Run
//! together they cover both failure modes.
//!
//! Adding a flag to `tree` means adding one row to each table below, not
//! writing a new test function.

use glob::Pattern;
use rstest::rstest;
use rust_tree::rust_tree::options::TreeOptions;
use rust_tree::rust_tree::traversal::list_directory_as_string;
use std::fs;
use std::path::PathBuf;
use tempfile::{tempdir, TempDir};

/// One logical tree, materialised twice: as real directories and as a listing.
///
/// The listing file deliberately lives *outside* `root` so the filesystem
/// renderer does not see it as an extra entry.
struct Fixture {
    _tmp: TempDir,
    root: PathBuf,
    listing: PathBuf,
}

/// Shaped so that every flag under test actually changes something:
/// a file sorts between two directories (`--dirsfirst`), the tree is three
/// levels deep (`-L`), and `many/` holds four children (`--filelimit`).
const ENTRIES: &[(&str, bool)] = &[
    ("adir", true),
    ("adir/x.rs", false),
    ("b.txt", false),
    ("cdir", true),
    ("cdir/deep", true),
    ("cdir/deep/z.rs", false),
    ("cdir/y.rs", false),
    ("many", true),
    ("many/f1.txt", false),
    ("many/f2.txt", false),
    ("many/f3.txt", false),
    ("many/f4.txt", false),
    ("zz.log", false),
];

fn fixture() -> Fixture {
    let tmp = tempdir().unwrap();
    let root = tmp.path().join("root");
    fs::create_dir(&root).unwrap();

    let mut listing = String::new();
    for (rel, is_dir) in ENTRIES {
        if *is_dir {
            fs::create_dir_all(root.join(rel)).unwrap();
            listing.push_str(&format!("{rel}/\n"));
        } else {
            // Distinct sizes so `-s` has something to render in filesystem mode.
            fs::write(root.join(rel), "x".repeat(rel.len())).unwrap();
            listing.push_str(&format!("{rel}\n"));
        }
    }

    let listing_path = tmp.path().join("listing.txt");
    fs::write(&listing_path, listing).unwrap();

    Fixture {
        _tmp: tmp,
        root,
        listing: listing_path,
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Mode {
    Filesystem,
    FromFile,
}

fn apply(options: &mut TreeOptions, flag: &str) {
    match flag {
        "" => {}
        "-r" => options.reverse = true,
        "--dirsfirst" => options.dirs_first = true,
        "-r --dirsfirst" => {
            options.reverse = true;
            options.dirs_first = true;
        }
        "-F" => options.classify = true,
        "-A" => options.ascii = true,
        "--no-indent" => options.no_indent = true,
        "-L 2" => options.level = Some(2),
        "-d" => options.dir_only = true,
        "-f" => options.full_path = true,
        "-s" => options.print_size = true,
        "-C" => options.color = true,
        "--icons" => options.icons = true,
        "--filelimit 3" => options.file_limit = Some(3),
        "-P *.rs" => options.pattern_glob = vec![Pattern::new("*.rs").unwrap()],
        "-I *.log" => options.exclude_patterns = vec![Pattern::new("*.log").unwrap()],
        "--prune -P *.rs" => {
            options.prune = true;
            options.pattern_glob = vec![Pattern::new("*.rs").unwrap()];
        }
        other => panic!("harness has no mapping for flag {other:?}"),
    }
}

fn render(fx: &Fixture, mode: Mode, flag: &str) -> String {
    let mut options = TreeOptions {
        no_report: true,
        ..Default::default()
    };
    let target = match mode {
        Mode::Filesystem => &fx.root,
        Mode::FromFile => {
            options.from_file = true;
            &fx.listing
        }
    };
    apply(&mut options, flag);
    list_directory_as_string(target, &options)
        .unwrap_or_else(|e| panic!("render failed for {mode:?} {flag:?}: {e}"))
}

/// Everything below the root line. The root label legitimately differs - the
/// filesystem renderer prints the directory path, the virtual one prints `.`.
fn body(output: &str) -> String {
    output
        .lines()
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string()
}

// ---------------------------------------------------------------------------
// Layer 1: does the flag do anything at all?
// ---------------------------------------------------------------------------

/// Which renderers a flag is expected to visibly affect.
#[derive(Clone, Copy)]
enum Effect {
    /// The flag must change output in both renderers.
    Both,
    /// The flag cannot affect `--fromfile` because a plain listing lacks the
    /// underlying data. The reason is recorded so the exemption stays honest.
    FilesystemOnly(&'static str),
}

/// A flag that leaves output byte-identical is doing nothing. This is the
/// shape of bug that made `--color`, `-r` and `--filelimit` silent no-ops
/// under `--fromfile` - each was "supported" but unobservable.
#[rstest]
#[case("-r", Effect::Both)]
#[case("--dirsfirst", Effect::Both)]
#[case("-F", Effect::Both)]
#[case("-A", Effect::Both)]
#[case("--no-indent", Effect::Both)]
#[case("-L 2", Effect::Both)]
#[case("-d", Effect::Both)]
#[case("-f", Effect::Both)]
#[case("-C", Effect::Both)]
#[case("--icons", Effect::Both)]
#[case("--filelimit 3", Effect::Both)]
#[case("-P *.rs", Effect::Both)]
#[case("-I *.log", Effect::Both)]
#[case("-s", Effect::FilesystemOnly("a plain path listing carries no sizes"))]
fn flag_changes_output(#[case] flag: &str, #[case] effect: Effect) {
    let fx = fixture();

    for mode in [Mode::Filesystem, Mode::FromFile] {
        let plain = render(&fx, mode, "");
        let flagged = render(&fx, mode, flag);

        let expect_change = !matches!((mode, effect), (Mode::FromFile, Effect::FilesystemOnly(_)));

        if expect_change {
            assert_ne!(
                plain, flagged,
                "{flag} is a silent no-op in {mode:?} mode - output is unchanged"
            );
        } else if let Effect::FilesystemOnly(why) = effect {
            assert_eq!(
                plain, flagged,
                "{flag} was exempted from {mode:?} mode because {why}, but it now \
                 changes output - promote this row to Effect::Both"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Layer 2: does the flag do the *same* thing in both renderers?
// ---------------------------------------------------------------------------

/// Whether the two renderers are expected to agree for a flag.
#[derive(Clone, Copy)]
enum Parity {
    /// Output below the root line must match byte for byte.
    Same,
    /// A divergence that exists today. The string documents why; when the
    /// underlying issue is fixed this row must become `Parity::Same`.
    Known(&'static str),
}

/// With no flags the two renderers already agree byte for byte below the root
/// line, which makes this a tight invariant rather than a loose smoke test.
///
/// Every `Known` row is a recorded defect, and the assertion is inverted for
/// them: fixing the defect fails this test and tells you to update the table.
#[rstest]
#[case("", Parity::Same)]
#[case("-r", Parity::Same)]
#[case("--dirsfirst", Parity::Same)]
#[case("-r --dirsfirst", Parity::Same)]
#[case("-F", Parity::Same)]
#[case("--no-indent", Parity::Same)]
#[case("-L 2", Parity::Same)]
#[case("-d", Parity::Same)]
#[case("-C", Parity::Same)]
#[case("--icons", Parity::Same)]
#[case("--filelimit 3", Parity::Same)]
#[case("-P *.rs", Parity::Same)]
#[case("-I *.log", Parity::Same)]
#[case("--prune -P *.rs", Parity::Same)]
#[case("-A", Parity::Same)]
#[case(
    "-s",
    Parity::Known(
        "a plain path listing carries no sizes, so the virtual renderer prints \
         none. Size-bearing listings additionally disagree on placement: virtual \
         prepends '[123]  ', filesystem appends ' [  123B]'."
    )
)]
#[case(
    "-f",
    Parity::Known(
        "the filesystem renderer prints an absolute path; a listing only ever \
         knows listing-relative paths. Inherent to --fromfile, not a defect."
    )
)]
fn renderers_agree(#[case] flag: &str, #[case] expected: Parity) {
    let fx = fixture();
    let filesystem = body(&render(&fx, Mode::Filesystem, flag));
    let from_file = body(&render(&fx, Mode::FromFile, flag));

    match expected {
        Parity::Same => assert_eq!(
            filesystem, from_file,
            "renderers disagree for {flag:?}\n--- filesystem ---\n{filesystem}\n\
             --- fromfile ---\n{from_file}\n"
        ),
        Parity::Known(why) => assert_ne!(
            filesystem, from_file,
            "{flag:?} is recorded as a known divergence but the renderers now \
             agree. If this was fixed deliberately, change the row to \
             Parity::Same. Recorded reason: {why}"
        ),
    }
}
