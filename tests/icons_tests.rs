use rust_tree::rust_tree::icons::IconManager;
use std::io::Write;
use std::path::Path;
use tempfile::{tempdir, NamedTempFile};

#[test]
fn test_get_icon_for_path() {
    let temp_dir = tempdir().unwrap();
    let mgr = IconManager::new();

    let cases = [
        ("cargo.toml", false),
        ("test.rs", false),
        ("unknown.xyz123", false),
    ];

    for (path, _) in cases {
        let icon = mgr.get_icon_for_path(Path::new(path));
        assert!(!icon.is_empty(), "icon for {path} should not be empty");
    }

    let icon = mgr.get_icon_for_path(temp_dir.path());
    assert!(!icon.is_empty(), "directory icon should not be empty");
}

#[test]
fn test_from_file() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(
        temp,
        r#"{{"well_known":{{"test.custom":"X"}},"extensions":{{}},"icons":{{"file":"F","dir":"D"}}}}"#
    )
    .unwrap();

    let mgr = IconManager::from_file(temp.path().to_str().unwrap()).unwrap();
    let icon = mgr.get_icon_for_path(Path::new("test.custom"));
    assert_eq!(icon, "X");
}
