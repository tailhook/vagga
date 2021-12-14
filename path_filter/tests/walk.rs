use std::path::{Path, PathBuf};

use crate::path_filter::PathFilter;


fn glob_paths(rules: &[&str]) -> Vec<PathBuf> {
    let path_filter = PathFilter::glob(rules).unwrap();
    let mut errors = vec!();
    let mut paths = {
        path_filter.walk_iterator("tests/dir1", &mut errors)
            .map(|e| e.path().to_path_buf())
            .collect::<Vec<_>>()
    };
    assert!(errors.is_empty());
    paths.sort();
    paths
}

fn regex_paths(ignore: Option<&str>, include: Option<&str>)
    -> Vec<PathBuf>
{
    let path_filter = PathFilter::regex(ignore, include).unwrap();
    let mut errors = vec!();
    let mut paths = {
        path_filter.walk_iterator("tests/dir1", &mut errors)
            .map(|e| e.path().to_path_buf())
            .collect::<Vec<_>>()
    };
    assert!(errors.is_empty());
    paths.sort();
    paths
}

#[test]
fn test_walk_glob() {
    assert_eq!(glob_paths(&[]).len(), 0);

    assert_eq!(glob_paths(&[""]).len(), 0);

    assert_eq!(glob_paths(&["/"]),
               vec!(Path::new("tests/dir1/dir/subdir/test.ini"),
                    Path::new("tests/dir1/dir/test.py"),
                    Path::new("tests/dir1/dir/test.rs"),
                    Path::new("tests/dir1/test.py"),
                    Path::new("tests/dir1/test.rs")));

    assert_eq!(glob_paths(&["*.rs"]),
               vec!(Path::new("tests/dir1/dir/test.rs"),
                    Path::new("tests/dir1/test.rs")));

    assert_eq!(glob_paths(&["/*.rs"]),
               vec!(Path::new("tests/dir1/test.rs")));

    assert_eq!(glob_paths(&["/dir/*.rs"]),
               vec!(Path::new("tests/dir1/dir/test.rs")));

    assert_eq!(glob_paths(&["dir/"]),
               vec!(Path::new("tests/dir1/dir"),
                    Path::new("tests/dir1/dir/subdir/test.ini"),
                    Path::new("tests/dir1/dir/test.py"),
                    Path::new("tests/dir1/dir/test.rs")));

    assert_eq!(glob_paths(&["/dir"]),
               vec!(Path::new("tests/dir1/dir/subdir/test.ini"),
                    Path::new("tests/dir1/dir/test.py"),
                    Path::new("tests/dir1/dir/test.rs")));

    assert_eq!(glob_paths(&["subdir/"]),
               vec!(Path::new("tests/dir1/dir/subdir"),
                    Path::new("tests/dir1/dir/subdir/test.ini")));

    assert_eq!(glob_paths(&["/**/*.rs"]),
               vec!(Path::new("tests/dir1/dir/test.rs"),
                    Path::new("tests/dir1/test.rs")));

    assert_eq!(glob_paths(&["!dir/", "*.rs"]),
               vec!(Path::new("tests/dir1/test.rs")));

    assert_eq!(glob_paths(&["!dir/subdir", "*.rs"]),
               vec!(Path::new("tests/dir1/dir/test.rs"),
                    Path::new("tests/dir1/test.rs")));

    assert_eq!(glob_paths(&["!subdir", "*.rs"]),
               vec!(Path::new("tests/dir1/dir/test.rs"),
                    Path::new("tests/dir1/test.rs")));

    assert_eq!(glob_paths(&["!*.rs", "/dir"]),
               vec!(Path::new("tests/dir1/dir/subdir/test.ini"),
                    Path::new("tests/dir1/dir/test.py")));

    assert_eq!(glob_paths(&["!*.rs", "!subdir", "/dir"]),
               vec!(Path::new("tests/dir1/dir/test.py")));

    assert_eq!(glob_paths(&["!dir/", "dir/"]),
               vec!(Path::new("tests/dir1/dir"),
                    Path::new("tests/dir1/dir/subdir/test.ini"),
                    Path::new("tests/dir1/dir/test.py"),
                    Path::new("tests/dir1/dir/test.rs")));
}

#[test]
fn test_walk_regex() {
    assert_eq!(regex_paths(None, Some(r".*\.rs")),
               vec!(Path::new("tests/dir1/dir/test.rs"),
                    Path::new("tests/dir1/test.rs")));

    assert_eq!(regex_paths(None, Some(r"(^|/)dir($|/)")),
               vec!(Path::new("tests/dir1/dir"),
                    Path::new("tests/dir1/dir/subdir"),
                    Path::new("tests/dir1/dir/subdir/test.ini"),
                    Path::new("tests/dir1/dir/test.py"),
                    Path::new("tests/dir1/dir/test.rs")));

    assert_eq!(regex_paths(Some(r"(^|/)dir($|/)"), Some(r".*\.rs")),
               vec!(Path::new("tests/dir1/test.rs")));
}
